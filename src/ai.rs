use crate::i18n::{msg, tr};
use crate::model::{Document, History, validate_text};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Codex,
    Claude,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CliSettings {
    pub codex: String,
    pub claude: String,
    pub provider: Provider,
    pub model: String,
    pub timeout_secs: u64,
    pub system_prompt: String,
    pub codex_options: CodexOptions,
    pub claude_options: ClaudeOptions,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CodexOptions {
    pub model: String,
    pub effort: String,
    pub service_tier: String,
    pub web_search: bool,
    pub oss: bool,
    pub local_provider: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeOptions {
    pub model: String,
    pub effort: String,
    pub fallback_model: String,
    pub max_budget_usd: String,
    pub add_dirs: String,
    pub disable_slash_commands: bool,
}
impl CliSettings {
    pub fn migrate_model(&mut self) {
        if !self.model.is_empty() {
            let target = match self.provider {
                Provider::Codex => &mut self.codex_options.model,
                Provider::Claude => &mut self.claude_options.model,
            };
            if target.is_empty() {
                *target = self.model.clone();
            }
            self.model.clear();
        }
    }
    pub fn active_model(&self) -> &str {
        let model = match self.provider {
            Provider::Codex => &self.codex_options.model,
            Provider::Claude => &self.claude_options.model,
        };
        if model.is_empty() { &self.model } else { model }
    }
    pub fn set_language(&mut self, language: &str) {
        if ["ja", "en"]
            .iter()
            .any(|lang| self.system_prompt == default_system_prompt_for(lang))
        {
            self.system_prompt = default_system_prompt_for(language);
        }
    }
}
impl Default for CliSettings {
    fn default() -> Self {
        Self {
            codex: detect("codex").unwrap_or_else(|| "codex.exe".into()),
            claude: detect("claude").unwrap_or_else(|| "claude.exe".into()),
            provider: Provider::Codex,
            model: String::new(),
            timeout_secs: 300,
            system_prompt: default_system_prompt(),
            codex_options: CodexOptions::default(),
            claude_options: ClaudeOptions::default(),
        }
    }
}
pub fn default_system_prompt() -> String {
    default_system_prompt_for(crate::i18n::language())
}
pub fn default_system_prompt_for(language: &str) -> String {
    (if language == "en" {
        include_str!("../assets/prompts/tree-edit.txt")
    } else {
        include_str!("../assets/prompts/tree-edit-ja.txt")
    })
    .trim()
    .to_owned()
}
/// Optional arguments are separate tokens, never shell command text.
pub fn optional_args(settings: &CliSettings) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut value = |flag: &str, text: &str| {
        if !text.trim().is_empty() {
            args.extend([flag.to_owned(), text.trim().to_owned()]);
        }
    };
    value("--model", settings.active_model());
    match settings.provider {
        Provider::Codex => {
            let opt = &settings.codex_options;
            value(
                "-c",
                &format!(
                    "web_search={}",
                    serde_json::to_string(if opt.web_search { "live" } else { "disabled" })?
                ),
            );
            if !opt.effort.is_empty() {
                value(
                    "-c",
                    &format!(
                        "model_reasoning_effort={}",
                        serde_json::to_string(&opt.effort)?
                    ),
                );
            }
            if !opt.service_tier.is_empty() {
                value(
                    "-c",
                    &format!("service_tier={}", serde_json::to_string(&opt.service_tier)?),
                );
            }
            if opt.oss {
                value("--local-provider", &opt.local_provider);
                args.push("--oss".into());
            }
        }
        Provider::Claude => {
            let opt = &settings.claude_options;
            value("--effort", &opt.effort);
            value("--fallback-model", &opt.fallback_model);
            if !opt.max_budget_usd.trim().is_empty() {
                let budget: f64 = opt
                    .max_budget_usd
                    .trim()
                    .parse()
                    .context(tr("cli.budget_invalid"))?;
                ensure!(
                    budget.is_finite() && budget > 0.,
                    "{}",
                    tr("cli.budget_invalid")
                );
                value("--max-budget-usd", &opt.max_budget_usd);
            }
            for dir in opt.add_dirs.lines().filter(|s| !s.trim().is_empty()) {
                value("--add-dir", dir);
            }
            if opt.disable_slash_commands {
                args.push("--disable-slash-commands".into());
            }
        }
    }
    Ok(args)
}
pub fn detect(name: &str) -> Option<String> {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join(&file);
        if p.is_file() {
            return Some(p.to_string_lossy().into());
        }
    }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    let home = PathBuf::from(home);
    for p in [
        home.join(".local/bin").join(&file),
        home.join("AppData/Local/Programs/OpenAI/Codex/bin")
            .join(&file),
        home.join(".cargo/bin").join(&file),
    ] {
        if p.is_file() {
            return Some(p.to_string_lossy().into());
        }
    }
    None
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Add {
        id: String,
        parent: String,
        text: String,
    },
    Update {
        id: String,
        text: String,
        note: Option<String>,
        details: Option<String>,
    },
    Delete {
        id: String,
    },
    Move {
        id: String,
        parent: String,
        index: usize,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditPlan {
    pub operations: Vec<Operation>,
}
#[derive(Clone)]
pub struct Snapshot {
    pub root: String,
    pub fingerprint: String,
    pub document_root: String,
    pub payload: String,
}
impl Snapshot {
    pub fn capture(doc: &Document, root: &str, instruction: &str) -> Result<Self> {
        doc.ensure_branch_editable(root)?;
        let ids = doc.subtree(root);
        let nodes:Vec<_>=ids.iter().map(|id|{let n=&doc.nodes[id];serde_json::json!({"id":id,"parent":if id==root{None}else{n.parent.as_deref()},"children":n.children,"text":n.text.text(),"note":n.note.text(),"details":n.details.text()})}).collect();
        Ok(Self {
            root: root.into(),
            fingerprint: doc.fingerprint(root),
            document_root: doc.root.clone(),
            payload: serde_json::to_string(
                &serde_json::json!({"instruction":instruction,"root":root,"nodes":nodes}),
            )?,
        })
    }
}
pub fn apply(history: &mut History, snapshot: &Snapshot, plan: &EditPlan) -> Result<usize> {
    ensure!(
        history.doc.root == snapshot.document_root,
        "{}",
        msg("ai.021", &[])
    );
    ensure!(
        history.doc.nodes.contains_key(&snapshot.root),
        "{}",
        msg("ai.020", &[])
    );
    ensure!(
        history.doc.fingerprint(&snapshot.root) == snapshot.fingerprint,
        "{}",
        msg("ai.019", &[])
    );
    ensure!(plan.operations.len() <= 5000, "{}", msg("ai.018", &[]));
    let scope = history.doc.subtree(&snapshot.root);
    let mut allowed = scope;
    history.edit(tr("ai.026"), |doc| {
        for op in &plan.operations {
            match op {
                Operation::Add { id, parent, text } => {
                    ensure!(
                        allowed.contains(parent) && doc.nodes.contains_key(parent),
                        "{}",
                        msg("ai.017", &[])
                    );
                    ensure!(
                        !doc.nodes.contains_key(id) && !allowed.contains(id),
                        "{}",
                        msg("ai.016", &[])
                    );
                    ensure!(!id.is_empty() && id.len() < 256, "{}", msg("ai.015", &[]));
                    validate_text(text)?;
                    let generated = doc.add(parent, text)?;
                    let mut n = doc.nodes.remove(&generated).unwrap();
                    n.id = id.clone();
                    doc.nodes
                        .get_mut(parent)
                        .unwrap()
                        .children
                        .iter_mut()
                        .filter(|c| **c == generated)
                        .for_each(|c| *c = id.clone());
                    doc.nodes.insert(id.clone(), n);
                    allowed.insert(id.clone());
                }
                Operation::Update {
                    id,
                    text,
                    note,
                    details,
                } => {
                    ensure!(allowed.contains(id), "{}", msg("ai.014", &[]));
                    doc.ensure_editable(id)?;
                    validate_text(text)?;
                    let n = doc.nodes.get_mut(id).unwrap();
                    ensure!(!n.text.unsupported, "{}", msg("ai.013", &[]));
                    n.text.edit(text);
                    if let Some(note) = note {
                        validate_text(note)?;
                        ensure!(!n.note.unsupported, "{}", msg("ai.012", &[]));
                        n.note.edit(note);
                    }
                    if let Some(details) = details {
                        validate_text(details)?;
                        ensure!(!n.details.unsupported, "{}", msg("ai.011", &[]));
                        n.details.edit(details);
                    }
                }
                Operation::Delete { id } => {
                    ensure!(
                        allowed.contains(id) && id != &snapshot.root,
                        "{}",
                        msg("ai.010", &[])
                    );
                    doc.delete(id)?;
                }
                Operation::Move { id, parent, index } => {
                    ensure!(
                        allowed.contains(id) && allowed.contains(parent) && id != &snapshot.root,
                        "{}",
                        msg("ai.009", &[])
                    );
                    doc.move_node(id, parent, *index)?;
                }
            }
        }
        Ok(())
    })?;
    Ok(plan.operations.len())
}
pub fn schema() -> serde_json::Value {
    fn object(properties: serde_json::Value, required: &[&str]) -> serde_json::Value {
        serde_json::json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    }
    let s = serde_json::json!({"type":"string"});
    let optional = serde_json::json!({"type":["string","null"]});
    object(
        serde_json::json!({"operations":{"type":"array","items":{"anyOf":[
            object(serde_json::json!({"op":{"const":"add","type":"string"},"id":s,"parent":s,"text":s}),&["op","id","parent","text"]),
            object(serde_json::json!({"op":{"const":"update","type":"string"},"id":s,"text":s,"note":optional,"details":optional}),&["op","id","text","note","details"]),
            object(serde_json::json!({"op":{"const":"delete","type":"string"},"id":s}),&["op","id"]),
            object(serde_json::json!({"op":{"const":"move","type":"string"},"id":s,"parent":s,"index":{"type":"integer","minimum":0}}),&["op","id","parent","index"])
        ]}}}),
        &["operations"],
    )
}
pub struct Task {
    pub snapshot: Snapshot,
    pub tab_id: String,
    pub cancel: Arc<AtomicBool>,
    pub receiver: mpsc::Receiver<Result<EditPlan, String>>,
    pub started: Instant,
}
pub fn spawn(
    settings: CliSettings,
    snapshot: Snapshot,
    tab_id: String,
    work: &Path,
    ctx: eframe::egui::Context,
) -> Task {
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let snap = snapshot.clone();
    let work = work.to_owned();
    let (tx, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result = execute(&settings, &snap, &work, &flag).map_err(|e| format!("{e:#}"));
        let _ = tx.send(result);
        ctx.request_repaint();
    });
    Task {
        snapshot,
        tab_id,
        cancel,
        receiver,
        started: Instant::now(),
    }
}
fn read_capped(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut data = vec![];
    let mut b = [0u8; 8192];
    let mut too_large = false;
    loop {
        let n = reader.read(&mut b)?;
        if n == 0 {
            break;
        }
        if data.len() + n <= 16 * 1024 * 1024 {
            data.extend_from_slice(&b[..n]);
        } else {
            too_large = true;
        }
    }
    ensure!(!too_large, "{}", msg("ai.008", &[]));
    Ok(data)
}
fn execute(
    settings: &CliSettings,
    snapshot: &Snapshot,
    work: &Path,
    cancel: &AtomicBool,
) -> Result<EditPlan> {
    let folder = work.join(format!("cli-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&folder)?;
    let result = (|| -> Result<EditPlan> {
        let schema_file = folder.join("schema.json");
        std::fs::write(&schema_file, serde_json::to_vec(&schema())?)?;
        let executable = match settings.provider {
            Provider::Codex => &settings.codex,
            Provider::Claude => &settings.claude,
        };
        let mut cmd = Command::new(executable);
        cmd.current_dir(&folder)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match settings.provider {
            Provider::Codex => {
                cmd.args([
                    "exec",
                    "--ignore-user-config",
                    "--skip-git-repo-check",
                    "--sandbox",
                    "read-only",
                    "--ephemeral",
                    "--color",
                    "never",
                    "--output-schema",
                ])
                .arg(&schema_file)
                .arg("-");
            }
            Provider::Claude => {
                cmd.args(["-p", "--output-format", "json", "--json-schema"])
                    .arg(schema().to_string())
                    .args([
                        "--tools",
                        "Read,Glob,Grep",
                        "--allowedTools",
                        "Read,Glob,Grep",
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
        cmd.args(optional_args(settings)?);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let mut child = cmd.spawn().context(tr("ai.025"))?;
        #[cfg(windows)]
        let _job = match ProcessJob::attach(&child) {
            Ok(job) => job,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        };
        let mut input = child.stdin.take().unwrap();
        let payload = format!(
            "{}\n\n<user_request_and_subtree_json>\n{}\n</user_request_and_subtree_json>",
            settings.system_prompt, snapshot.payload
        );
        let writer = std::thread::spawn(move || input.write_all(payload.as_bytes()));
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || read_capped(stdout));
        let err = std::thread::spawn(move || read_capped(stderr));
        let start = Instant::now();
        let status = loop {
            if cancel.load(Ordering::Relaxed)
                || start.elapsed() > Duration::from_secs(settings.timeout_secs.max(1))
            {
                #[cfg(windows)]
                drop(_job);
                let _ = child.kill();
                let _ = child.wait();
                let _ = writer.join();
                let _ = out.join();
                let _ = err.join();
                bail!(if cancel.load(Ordering::Relaxed) {
                    tr("ai.024")
                } else {
                    tr("ai.023")
                });
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        writer
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.007", &[])))??;
        let output = out
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.006", &[])))??;
        let error = err
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.005", &[])))??;
        if !status.success() {
            let detail = if error.is_empty() {
                serde_json::from_slice::<serde_json::Value>(&output)
                    .ok()
                    .and_then(|v| v.get("result").and_then(|s| s.as_str()).map(str::to_owned))
                    .unwrap_or_else(|| String::from_utf8_lossy(&output).into_owned())
            } else {
                String::from_utf8_lossy(&error).into_owned()
            };
            bail!(
                "{}",
                msg(
                    "ai.004",
                    &[(
                        "0",
                        detail.chars().take(2000).collect::<String>().to_string()
                    )]
                )
            );
        }
        parse_output(settings.provider, &output)
    })();
    let _ = std::fs::remove_file(folder.join("schema.json"));
    let _ = std::fs::remove_dir(&folder);
    result
}
pub fn parse_output(provider: Provider, bytes: &[u8]) -> Result<EditPlan> {
    let json: serde_json::Value = serde_json::from_slice(bytes).context(tr("ai.022"))?;
    if provider == Provider::Claude {
        ensure!(
            !json
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "Claude Code: {}",
            json.get("result").unwrap_or(&json)
        );
        if let Some(s) = json.get("structured_output") {
            return Ok(serde_json::from_value(s.clone())?);
        }
        if let Some(s) = json.get("result").and_then(|v| v.as_str()) {
            return Ok(serde_json::from_str(s)?);
        }
    }
    Ok(serde_json::from_value(json)?)
}
#[cfg(windows)]
pub(crate) struct ProcessJob(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl ProcessJob {
    pub(crate) fn attach(child: &std::process::Child) -> Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            ensure!(!h.is_null(), "{}", msg("ai.003", &[]));
            let job = Self(h);
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            ensure!(
                SetInformationJobObject(
                    h,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as _,
                    std::mem::size_of_val(&info) as u32
                ) != 0,
                "{}",
                msg("ai.002", &[])
            );
            ensure!(
                AssignProcessToJobObject(h, child.as_raw_handle() as _) != 0,
                "{}",
                msg("ai.001", &[])
            );
            Ok(job)
        }
    }
}
#[cfg(windows)]
impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;
    #[test]
    fn optional_arguments_are_separate_and_provider_specific() {
        let mut s = CliSettings {
            codex_options: CodexOptions {
                model: "custom model".into(),
                effort: "high".into(),
                service_tier: "fast".into(),
                web_search: true,
                oss: true,
                local_provider: "ollama".into(),
            },
            claude_options: ClaudeOptions {
                model: "sonnet".into(),
                effort: "max".into(),
                fallback_model: "haiku".into(),
                max_budget_usd: "1.25".into(),
                add_dirs: "C:/Knowledge Base".into(),
                disable_slash_commands: true,
            },
            ..Default::default()
        };
        let args = optional_args(&s).unwrap();
        assert!(args.windows(2).any(|a| a == ["--model", "custom model"]));
        assert!(args.contains(&"model_reasoning_effort=\"high\"".into()));
        assert!(args.contains(&"web_search=\"live\"".into()));
        assert!(args.contains(&"--oss".into()));
        assert!(!args.contains(&"--effort".into()));
        s.provider = Provider::Claude;
        let args = optional_args(&s).unwrap();
        assert!(args.windows(2).any(|a| a == ["--model", "sonnet"]));
        assert!(
            args.windows(2)
                .any(|a| a == ["--add-dir", "C:/Knowledge Base"])
        );
        assert!(args.contains(&"--disable-slash-commands".into()));
        assert!(!args.contains(&"--oss".into()));
        for invalid in ["-1", "NaN", "infinity", "invalid"] {
            s.claude_options.max_budget_usd = invalid.into();
            assert!(optional_args(&s).is_err());
        }
    }
    #[test]
    fn localized_defaults_switch_without_overwriting_custom_prompts() {
        let mut s = CliSettings {
            system_prompt: default_system_prompt_for("ja"),
            ..Default::default()
        };
        s.set_language("en");
        assert_eq!(s.system_prompt, default_system_prompt_for("en"));
        s.set_language("ja");
        assert_eq!(s.system_prompt, default_system_prompt_for("ja"));
        s.system_prompt = "User knowledge reference instructions".into();
        s.set_language("en");
        assert_eq!(s.system_prompt, "User knowledge reference instructions");
    }
    #[test]
    fn old_shared_model_migrates_to_selected_provider_only() {
        let mut s = CliSettings {
            model: "existing".into(),
            provider: Provider::Claude,
            ..Default::default()
        };
        s.migrate_model();
        assert_eq!(s.claude_options.model, "existing");
        assert!(s.codex_options.model.is_empty());
        assert!(s.model.is_empty());
    }
}
