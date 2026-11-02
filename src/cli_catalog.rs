//! Model metadata discovery never starts an inference turn or loads credentials itself.
use crate::{
    ai::{CliSettings, Provider},
    i18n::tr,
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelChoice {
    pub id: String,
    pub label: String,
    pub efforts: Vec<String>,
    pub is_default: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub models: Vec<ModelChoice>,
    pub configured_model: String,
    pub configured_effort: String,
    pub source: String,
    pub warning: String,
}
fn home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
}
pub fn codex_home() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".codex"))
}
pub fn claude_home() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".claude"))
}
pub fn parse_models(provider: Provider, data: &Value) -> Vec<ModelChoice> {
    let Some(items) = data.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter(|v| {
            !v["hidden"].as_bool().unwrap_or(false) && v["visibility"].as_str() != Some("hide")
        })
        .filter_map(|v| {
            let id = match provider {
                Provider::Codex => v["model"].as_str().or(v["slug"].as_str()),
                Provider::Claude => v["value"].as_str(),
            }?;
            let efforts = v["supportedReasoningEfforts"]
                .as_array()
                .or(v["supported_reasoning_levels"].as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| {
                            v["reasoningEffort"]
                                .as_str()
                                .or(v["effort"].as_str())
                                .map(str::to_owned)
                        })
                        .collect()
                })
                .or_else(|| {
                    v["supportedEffortLevels"].as_array().map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                })
                .unwrap_or_default();
            Some(ModelChoice {
                id: id.into(),
                label: v["displayName"]
                    .as_str()
                    .or(v["display_name"].as_str())
                    .unwrap_or(id)
                    .into(),
                efforts,
                is_default: v["isDefault"].as_bool().unwrap_or(false) || id == "default",
            })
        })
        .collect()
}
pub fn local(provider: Provider) -> Catalog {
    let mut result = Catalog::default();
    match provider {
        Provider::Codex => {
            let dir = codex_home();
            if let Ok(text) = std::fs::read_to_string(dir.join("config.toml")) {
                if let Ok(doc) = text.parse::<toml_edit::DocumentMut>() {
                    result.configured_model = doc
                        .get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .into();
                    result.configured_effort = doc
                        .get("model_reasoning_effort")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .into();
                } else {
                    result.warning = tr("cli.config_invalid").into();
                }
            }
            if let Ok(bytes) = std::fs::read(dir.join("models_cache.json"))
                && let Ok(data) = serde_json::from_slice::<Value>(&bytes)
            {
                result.models = parse_models(provider, &data["models"]);
            }
            result.source = tr("cli.source_codex_local").into();
        }
        Provider::Claude => {
            if let Ok(bytes) = std::fs::read(claude_home().join("settings.json")) {
                if let Ok(data) = serde_json::from_slice::<Value>(&bytes) {
                    result.configured_model = data["model"].as_str().unwrap_or_default().into();
                    result.configured_effort =
                        data["effortLevel"].as_str().unwrap_or_default().into();
                    if let Some(models) = data["availableModels"].as_array() {
                        result.models = models
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|id| ModelChoice {
                                id: id.into(),
                                label: id.into(),
                                efforts: Vec::new(),
                                is_default: false,
                            })
                            .collect();
                    }
                } else {
                    result.warning = tr("cli.config_invalid").into();
                }
            }
            if let Ok(model) = std::env::var("ANTHROPIC_MODEL") {
                result.configured_model = model;
            }
            if let Ok(effort) = std::env::var("CLAUDE_CODE_EFFORT_LEVEL") {
                result.configured_effort = effort;
            }
            result.source = tr("cli.source_claude_local").into();
        }
    }
    if !result.configured_model.is_empty()
        && !result
            .models
            .iter()
            .any(|m| m.id == result.configured_model)
    {
        result.models.push(ModelChoice {
            id: result.configured_model.clone(),
            label: result.configured_model.clone(),
            efforts: Vec::new(),
            is_default: false,
        });
    }
    result
}
/// A short-lived stdio control handshake only. Child termination bounds unsupported CLI versions.
fn remote(settings: &CliSettings, work: &Path) -> Result<Vec<ModelChoice>> {
    let mut cmd = Command::new(match settings.provider {
        Provider::Codex => &settings.codex,
        Provider::Claude => &settings.claude,
    });
    match settings.provider {
        Provider::Codex => {
            cmd.args(["app-server", "--stdio"]);
        }
        Provider::Claude => {
            cmd.args([
                "-p",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
                "--tools",
                "",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--setting-sources",
                "",
                "--settings",
                "{\"disableAllHooks\":true}",
                "--no-session-persistence",
            ]);
        }
    }
    cmd.current_dir(work)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let mut child = cmd.spawn().context(tr("cli.discovery_failed"))?;
    #[cfg(windows)]
    let _job = match crate::ai::ProcessJob::attach(&child) {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let mut input = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::sync_channel(32);
    let reader = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match (&mut reader).take(2 * 1024 * 1024).read_line(&mut line) {
                Ok(0) | Err(_) => break,
                _ => {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let result = (|| -> Result<Vec<ModelChoice>> {
        let init = match settings.provider {
            Provider::Codex => {
                json!({"id":0,"method":"initialize","params":{"clientInfo":{"name":"mindglider","title":"MindGlider","version":env!("CARGO_PKG_VERSION")}}})
            }
            Provider::Claude => {
                json!({"type":"control_request","request_id":"init","request":{"subtype":"initialize"}})
            }
        };
        writeln!(input, "{init}")?;
        input.flush()?;
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut models = Vec::new();
        loop {
            let line = rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|e| {
                    anyhow::anyhow!(
                        "{}",
                        match e {
                            mpsc::RecvTimeoutError::Timeout => tr("cli.discovery_timeout"),
                            mpsc::RecvTimeoutError::Disconnected => tr("cli.discovery_failed"),
                        }
                    )
                })?;
            let Ok(v) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            match settings.provider {
                Provider::Codex => {
                    if v["id"] == 0 {
                        ensure!(v.get("error").is_none(), "{}", tr("cli.discovery_failed"));
                        writeln!(input, "{}", json!({"method":"initialized","params":{}}))?;
                        writeln!(
                            input,
                            "{}",
                            json!({"id":1,"method":"model/list","params":{"limit":100,"includeHidden":false}})
                        )?;
                        input.flush()?;
                    } else if v["id"] == 1 {
                        ensure!(v.get("error").is_none(), "{}", tr("cli.discovery_failed"));
                        models.extend(parse_models(settings.provider, &v["result"]["data"]));
                        if let Some(cursor) = v["result"]["nextCursor"].as_str() {
                            writeln!(
                                input,
                                "{}",
                                json!({"id":1,"method":"model/list","params":{"limit":100,"includeHidden":false,"cursor":cursor}})
                            )?;
                            input.flush()?;
                        } else {
                            return Ok(models);
                        }
                    }
                }
                Provider::Claude => {
                    if v["type"] == "control_response" && v["response"]["request_id"] == "init" {
                        ensure!(
                            v["response"]["subtype"] == "success",
                            "{}",
                            tr("cli.discovery_failed")
                        );
                        return Ok(parse_models(
                            settings.provider,
                            &v["response"]["response"]["models"],
                        ));
                    }
                }
            }
            if Instant::now() >= deadline {
                bail!("{}", tr("cli.discovery_timeout"));
            }
        }
    })();
    #[cfg(windows)]
    drop(_job);
    let _ = child.kill();
    let _ = child.wait();
    drop(input);
    drop(rx);
    let _ = reader.join();
    result
}
pub fn discover(settings: &CliSettings, work: &Path) -> Catalog {
    let mut catalog = local(settings.provider);
    match remote(settings, work) {
        Ok(models) if !models.is_empty() => {
            catalog.models = models;
            catalog.source = match settings.provider {
                Provider::Codex => tr("cli.source_codex_live"),
                Provider::Claude => tr("cli.source_claude_live"),
            }
            .into();
        }
        Ok(_) => catalog.warning = tr("cli.models_empty").into(),
        Err(e) => catalog.warning = format!("{e:#}"),
    }
    catalog
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_model_specific_efforts_and_hides_internal_models() {
        let data = json!([{"slug":"a","display_name":"A","supported_reasoning_levels":[{"effort":"low"},{"effort":"max"}]},{"slug":"b","visibility":"hide"}]);
        let c = parse_models(Provider::Codex, &data);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].efforts, ["low", "max"]);
        let c = parse_models(
            Provider::Claude,
            &json!([{"value":"sonnet","supportedEffortLevels":["low","high"]},{"value":"haiku"}]),
        );
        assert_eq!(c[0].efforts, ["low", "high"]);
        assert!(c[1].efforts.is_empty());
    }
}
