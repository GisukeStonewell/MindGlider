use anyhow::{Context, Result};
use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, ErrorChannel, FileSpec, Logger, LoggerHandle, Naming,
    filter::{LogLineFilter, LogLineWriter},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

const FATAL_TARGET: &str = "mindglider::fatal";
const BASENAME: &str = "mindglider";
static INCLUDE_RESPONSE_CONTENT: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Level {
    Off,
    Fatal,
    Error,
    #[serde(alias = "warn")]
    Warning,
    #[default]
    Info,
    Debug,
    Trace,
}

impl Level {
    fn specification(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Fatal => "off,mindglider::fatal=error",
            Self::Error => "off,mindglider=error",
            Self::Warning => "off,mindglider=warn",
            Self::Info => "off,mindglider=info",
            Self::Debug => "off,mindglider=debug",
            Self::Trace => "off,mindglider=trace",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Rotation {
    #[default]
    Daily,
    Never,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    level: Level,
    rotation: Rotation,
    retention_days: NonZeroU32,
    include_response_content: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            level: Level::Info,
            rotation: Rotation::Daily,
            retention_days: NonZeroU32::new(3).unwrap(),
            include_response_content: false,
        }
    }
}

fn load_settings(path: &Path) -> Result<Settings> {
    let bytes = match fs::read(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let defaults = Settings::default();
            let mut bytes = serde_json::to_vec_pretty(&defaults)?;
            bytes.push(b'\n');
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut file) => {
                    file.write_all(&bytes)?;
                    return Ok(defaults);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => fs::read(path)?,
                Err(error) => return Err(error.into()),
            }
        }
        result => result?,
    };
    // Accept the UTF-8 BOM added by some Windows editors.
    Ok(serde_json::from_slice(
        bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes),
    )?)
}

/// Start MindGlider's process-wide logger and retain the returned handle until shutdown.
pub fn init(data: &Path) -> Result<LoggerHandle> {
    fs::create_dir_all(data).with_context(|| format!("create {}", data.display()))?;
    let config = data.join("logging.json");
    let (settings, config_error) = match load_settings(&config) {
        Ok(settings) => (settings, None),
        Err(error) => (Settings::default(), Some(error)),
    };
    let directory = data.join("logs");
    let handle = build_logger(&directory, &settings)?
        .error_channel(ErrorChannel::File(data.join("logging-error.log")))
        .start()
        .with_context(|| format!("start logger in {}", directory.display()))?;
    INCLUDE_RESPONSE_CONTENT.store(settings.include_response_content, Ordering::Relaxed);
    if let Some(error) = config_error {
        log::warn!(
            "event=logging.config_invalid path={:?} error={error:#}",
            config
        );
    }
    Ok(handle)
}

pub fn include_response_content() -> bool {
    INCLUDE_RESPONSE_CONTENT.load(Ordering::Relaxed)
}

fn build_logger(directory: &Path, settings: &Settings) -> Result<Logger> {
    fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    let spec = FileSpec::default()
        .directory(directory)
        .basename(BASENAME)
        .suppress_timestamp();
    let logger = Logger::try_with_str(settings.level.specification())?
        .log_to_file(spec)
        .append()
        .format(format_line)
        .panic_if_error_channel_is_broken(false);
    if settings.rotation == Rotation::Never {
        return Ok(logger);
    }
    let retention = DailyRetention {
        directory: directory.to_owned(),
        days: settings.retention_days,
        last_checked_date: Mutex::new(String::new()),
    };
    retention.check(&mut DeferredNow::new());
    Ok(logger
        .rotate(
            Criterion::Age(Age::Day),
            Naming::TimestampsCustomFormat {
                current_infix: None,
                format: "%Y-%m-%d",
            },
            // DailyRetention uses calendar dates rather than elapsed 24-hour periods.
            Cleanup::Never,
        )
        .filter(Box::new(retention)))
}

fn format_line(out: &mut dyn Write, now: &mut DeferredNow, record: &log::Record) -> io::Result<()> {
    let level = if record.target() == FATAL_TARGET && record.level() == log::Level::Error {
        "FATAL"
    } else {
        record.level().as_str()
    };
    let mut message = record.args().to_string();
    if message.contains(['\r', '\n']) {
        message = message.replace('\r', "\\r").replace('\n', "\\n");
    }
    write!(
        out,
        "{} {} {} {}",
        now.format_rfc3339(),
        level,
        record.target(),
        message
    )
}

struct DailyRetention {
    directory: PathBuf,
    days: NonZeroU32,
    last_checked_date: Mutex<String>,
}

impl DailyRetention {
    fn check(&self, now: &mut DeferredNow) {
        let today = now.now().date_naive();
        let date_key = today.to_string();
        let mut last_checked = self
            .last_checked_date
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *last_checked == date_key {
            return;
        }
        *last_checked = date_key;
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return;
        };
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let Some(dated) = filename
                .to_str()
                .and_then(|name| name.strip_prefix("mindglider_"))
                .and_then(|name| name.strip_suffix(".log"))
            else {
                continue;
            };
            let date = match dated.split_once(".restart-") {
                Some((date, suffix))
                    if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    date
                }
                _ => dated,
            };
            if date.len() == 10
                && let Ok(file_date) = date.parse()
                && today.signed_duration_since(file_date).num_days() >= i64::from(self.days.get())
                && entry.file_type().is_ok_and(|kind| kind.is_file())
            {
                // Another process may rotate, remove, or still be holding this file.
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

impl LogLineFilter for DailyRetention {
    fn write(
        &self,
        now: &mut DeferredNow,
        record: &log::Record,
        writer: &dyn LogLineWriter,
    ) -> io::Result<()> {
        self.check(now);
        writer.write(now, record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level as LogLevel;

    #[test]
    fn absent_config_gets_defaults_and_existing_options_are_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logging.json");
        assert_eq!(load_settings(&path).unwrap(), Settings::default());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(
            serde_json::from_slice::<Settings>(&bytes).unwrap(),
            Settings::default()
        );
        let user_config = br#"{"level":"warn","rotation":"never","retention_days":7}"#;
        fs::write(&path, user_config).unwrap();
        let settings = load_settings(&path).unwrap();
        assert_eq!(settings.level, Level::Warning);
        assert_eq!(settings.rotation, Rotation::Never);
        assert_eq!(settings.retention_days.get(), 7);
        assert_eq!(fs::read(&path).unwrap(), user_config);
        fs::write(&path, br#"{"retention_days":0}"#).unwrap();
        assert!(load_settings(&path).is_err());
    }

    #[test]
    fn daily_logs_append_and_prune_only_expired_dated_files() {
        let dir = tempfile::tempdir().unwrap();
        let today = DeferredNow::new().now().date_naive();
        let yesterday = today.pred_opt().unwrap();
        let two_days_ago = yesterday.pred_opt().unwrap();
        let three_days_ago = two_days_ago.pred_opt().unwrap();
        let dated = |date| dir.path().join(format!("mindglider_{date}.log"));
        fs::write(dated(two_days_ago), "keep").unwrap();
        fs::write(dated(three_days_ago), "expire").unwrap();
        let expired_restart = dir
            .path()
            .join(format!("mindglider_{three_days_ago}.restart-0001.log"));
        fs::write(&expired_restart, "expire").unwrap();
        let unrelated = dir.path().join("unrelated.log");
        fs::write(&unrelated, "keep").unwrap();

        for text in ["first run", "second run"] {
            let (logger, handle) = build_logger(dir.path(), &Settings::default())
                .unwrap()
                .build()
                .unwrap();
            logger.log(
                &log::Record::builder()
                    .level(LogLevel::Info)
                    .target("mindglider::ai")
                    .args(format_args!("{}", text))
                    .build(),
            );
            handle.flush();
        }
        let log = fs::read_to_string(dated(today)).unwrap();
        assert!(log.contains(" INFO mindglider::ai first run"));
        assert!(log.contains(" INFO mindglider::ai second run"));
        assert!(dated(two_days_ago).is_file());
        assert!(!dated(three_days_ago).exists());
        assert!(!expired_restart.exists());
        assert!(unrelated.is_file());
    }

    #[test]
    fn fatal_threshold_and_single_line_format() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings {
            level: Level::Fatal,
            rotation: Rotation::Never,
            ..Settings::default()
        };
        let (logger, handle) = build_logger(dir.path(), &settings)
            .unwrap()
            .build()
            .unwrap();
        for (target, text) in [
            ("mindglider::ai", "ordinary"),
            (FATAL_TARGET, "fatal\nline"),
        ] {
            logger.log(
                &log::Record::builder()
                    .target(target)
                    .level(LogLevel::Error)
                    .args(format_args!("{}", text))
                    .build(),
            );
        }
        handle.flush();
        let line = fs::read_to_string(dir.path().join("mindglider.log")).unwrap();
        assert_eq!(line.lines().count(), 1);
        assert!(line.contains(" FATAL mindglider::fatal fatal\\nline"));
    }
}
