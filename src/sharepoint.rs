use anyhow::{Context, Result, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use mindglider::{i18n::tr, mm, model::Document};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::mpsc,
    time::Instant,
};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    tenant: uuid::Uuid,
    drive: String,
    item: String,
}

impl Location {
    pub fn parse(input: &str) -> Result<Self> {
        let url = Url::parse(input).context(tr("sharepoint.invalid"))?;
        ensure!(
            url.scheme() == "mindglider"
                && url.host_str() == Some("sharepoint")
                && matches!(url.path(), "" | "/")
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && url.fragment().is_none(),
            "{}",
            tr("sharepoint.invalid")
        );
        let mut tenant = None;
        let mut drive = None;
        let mut item = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "tenant" if tenant.is_none() => tenant = uuid::Uuid::parse_str(&value).ok(),
                "drive" if drive.is_none() => drive = Some(value.into_owned()),
                "item" if item.is_none() => item = Some(value.into_owned()),
                _ => anyhow::bail!("{}", tr("sharepoint.invalid")),
            }
            if key == "tenant" {
                ensure!(tenant.is_some(), "{}", tr("sharepoint.invalid"));
            }
        }
        let valid_id = |s: &str| {
            !s.is_empty()
                && s.len() <= 512
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"!-_".contains(&b))
        };
        Ok(Self {
            tenant: tenant
                .filter(|id| !id.is_nil())
                .context(tr("sharepoint.invalid"))?,
            drive: drive
                .filter(|s| valid_id(s))
                .context(tr("sharepoint.invalid"))?,
            item: item
                .filter(|s| valid_id(s))
                .context(tr("sharepoint.invalid"))?,
        })
    }
}

pub struct Version {
    pub account: String,
    pub tag: String,
    pub doc: Document,
}

pub enum Reply {
    Unchanged,
    Loaded(Box<Version>),
    Saved(String),
    Conflict,
    WriteDenied,
}

pub struct Session {
    pub location: Location,
    pub base: Version,
    pub due: Instant,
    pub job: Option<Job>,
    pub autosave: bool,
    pub manual_pending: bool,
    pub check_first: bool,
    pub conflict: Option<Version>,
    pub recovery_revision: Option<u64>,
}

pub struct Job {
    pub receive: mpsc::Receiver<Result<Reply>>,
    pub sent: Option<Document>,
}
impl Job {
    pub fn result(&self) -> Option<Result<Reply>> {
        match self.receive.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(anyhow::anyhow!(
                "{}",
                tr("sharepoint.connection_error")
            ))),
        }
    }
}

pub fn request(
    location: Location,
    parent: isize,
    ctx: eframe::egui::Context,
    base: Option<&Version>,
    sent: Option<Document>,
    interactive: bool,
) -> Result<Job> {
    let config: Config = serde_json::from_slice(
        &std::fs::read(mindglider::storage::data_dir().join("sharepoint.json"))
            .context(tr("sharepoint.configure"))?,
    )
    .context(tr("sharepoint.configure"))?;
    ensure!(
        config.tenant_id == location.tenant && !config.client_id.is_nil(),
        "{}",
        tr("sharepoint.configure")
    );
    let exe = std::env::current_exe()?
        .with_file_name("sharepoint")
        .join("MindGlider.SharePoint.exe");
    ensure!(exe.is_file(), "{}", tr("sharepoint.helper"));
    let bytes = sent.as_ref().map(mm::write).transpose()?;
    ensure!(
        bytes
            .as_ref()
            .is_none_or(|bytes| bytes.len() <= 16 * 1024 * 1024),
        "{}",
        tr("sharepoint.invalid_map")
    );
    let saving = bytes.is_some();
    let request = serde_json::json!({
        "Account": base.map(|v| &v.account),
        "Tag": base.map(|v| &v.tag),
        "Bytes": bytes.as_ref().map(|bytes| STANDARD.encode(bytes)),
        "Interactive": interactive,
    });
    let (send, receive) = mpsc::channel();
    std::thread::Builder::new()
        .name("sharepoint-sync".into())
        .spawn(move || {
            let result = (|| {
                let mut command = Command::new(exe);
                command.args([
                    &config.tenant_id.to_string(),
                    &config.client_id.to_string(),
                    &location.drive,
                    &location.item,
                    &parent.to_string(),
                ]);
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .context(tr("sharepoint.helper"))?;
                let input = child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(request.to_string().as_bytes());
                let output = child.wait_with_output().context(tr("sharepoint.helper"))?;
                if output.status.code() == Some(6) {
                    return Ok(Reply::Conflict);
                }
                if saving && output.status.code() == Some(3) {
                    return Ok(Reply::WriteDenied);
                }
                if !output.status.success() {
                    let key = match output.status.code() {
                        Some(2) => "sharepoint.auth_error",
                        Some(3) => "sharepoint.access_error",
                        Some(4) => "sharepoint.invalid_map",
                        Some(7) => "sharepoint.locked",
                        _ => "sharepoint.connection_error",
                    };
                    anyhow::bail!("{}", tr(key));
                }
                input?;
                let response: Response = serde_json::from_slice(&output.stdout)
                    .context(tr("sharepoint.connection_error"))?;
                match (saving, response.tag, response.bytes) {
                    (true, Some(tag), None) => Ok(Reply::Saved(tag)),
                    (false, None, None) => Ok(Reply::Unchanged),
                    (false, Some(tag), Some(bytes)) => Ok(Reply::Loaded(Box::new(Version {
                        account: response.account,
                        tag,
                        doc: mm::read(&STANDARD.decode(bytes)?)
                            .context(tr("sharepoint.invalid_map"))?,
                    }))),
                    _ => anyhow::bail!("{}", tr("sharepoint.connection_error")),
                }
            })();
            let _ = send.send(result);
            ctx.request_repaint();
        })?;
    Ok(Job { receive, sent })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    account: String,
    tag: Option<String>,
    bytes: Option<String>,
}
#[derive(serde::Deserialize)]
struct Config {
    #[serde(deserialize_with = "uuid_string")]
    tenant_id: uuid::Uuid,
    #[serde(deserialize_with = "uuid_string")]
    client_id: uuid::Uuid,
}

fn uuid_string<'de, D: serde::Deserializer<'de>>(de: D) -> Result<uuid::Uuid, D::Error> {
    let value = <String as serde::Deserialize>::deserialize(de)?;
    uuid::Uuid::parse_str(&value).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_explicit_sharepoint_item_links() {
        let valid = "mindglider://sharepoint?tenant=52d13ec3-244a-44b6-9072-19cfa0eaac05&drive=b!test&item=01TEST";
        assert!(Location::parse(valid).is_ok());
        for bad in [
            valid.replace("mindglider:", "https:"),
            valid.replace("b!test", "%2F..%2Ftest"),
            format!("{valid}&item=second"),
            format!("{valid}&token=secret"),
            format!("{valid}#fragment"),
        ] {
            assert!(Location::parse(&bad).is_err());
        }
    }
}
