//! Editable UTF-8 catalogs. Interned text keeps widget labels stable across a frame/reload.
use anyhow::{Context, Result};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

pub const JA: &str = include_str!("../locales/ja.json");
pub const EN: &str = include_str!("../locales/en.json");
pub fn directory() -> PathBuf {
    std::env::var_os("MINDGLIDER_LOCALES_DIR")
        .or_else(|| std::env::var_os(crate::compat::OLD_LOCALES_ENV))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("locales")
        })
}
fn builtin(language: &str) -> BTreeMap<String, String> {
    serde_json::from_str(if language == "en" { EN } else { JA })
        .expect("bundled catalog must be valid")
}
struct Runtime {
    language: String,
    tables: BTreeMap<String, BTreeMap<String, &'static str>>,
    pool: BTreeMap<String, &'static str>,
}
impl Runtime {
    fn new() -> Self {
        let mut r = Self {
            language: "ja".into(),
            tables: BTreeMap::new(),
            pool: BTreeMap::new(),
        };
        for lang in ["ja", "en"] {
            r.install(lang, builtin(lang));
        }
        r
    }
    fn install(&mut self, language: &str, table: BTreeMap<String, String>) {
        let table = table
            .into_iter()
            .map(|(key, text)| {
                let value = *self
                    .pool
                    .entry(text.clone())
                    .or_insert_with(|| Box::leak(text.into_boxed_str()));
                (key, value)
            })
            .collect();
        self.tables.insert(language.into(), table);
    }
}
static RUNTIME: OnceLock<RwLock<Runtime>> = OnceLock::new();
fn runtime() -> &'static RwLock<Runtime> {
    RUNTIME.get_or_init(|| RwLock::new(Runtime::new()))
}
pub fn language() -> &'static str {
    if runtime().read().unwrap().language == "en" {
        "en"
    } else {
        "ja"
    }
}
pub fn tr(key: &str) -> &'static str {
    let r = runtime().read().unwrap();
    r.tables
        .get(&r.language)
        .and_then(|t| t.get(key))
        .copied()
        .unwrap_or("[missing translation]")
}
pub fn msg(key: &str, args: &[(&str, String)]) -> String {
    render(tr(key), args)
}
pub fn render(template: &str, args: &[(&str, String)]) -> String {
    static PARAM: OnceLock<regex::Regex> = OnceLock::new();
    let pattern = PARAM.get_or_init(|| regex::Regex::new(r"\{([A-Za-z0-9_]+)\}").unwrap());
    pattern
        .replace_all(template, |c: &regex::Captures| {
            args.iter()
                .find(|(key, _)| *key == &c[1])
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| c[0].to_owned())
        })
        .into_owned()
}
pub fn initialize(language: &str) -> Result<()> {
    if matches!(language, "ja" | "en") {
        runtime().write().unwrap().language = language.into();
    }
    let dir = directory();
    std::fs::create_dir_all(&dir)?;
    for (name, text) in [("ja.json", JA), ("en.json", EN)] {
        let path = dir.join(name);
        if !path.exists() {
            use std::io::Write;
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => file.write_all(text.as_bytes())?,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
    reload(language)
}
pub fn reload(language: &str) -> Result<()> {
    anyhow::ensure!(matches!(language, "ja" | "en"), tr("locale.unsupported"));
    let path = directory().join(format!("{language}.json"));
    let text = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    let table = merge_overrides(language, &text).with_context(|| path.display().to_string())?;
    let mut r = runtime().write().unwrap();
    r.install(language, table);
    r.language = language.into();
    Ok(())
}

pub fn merge_overrides(language: &str, text: &str) -> Result<BTreeMap<String, String>> {
    let custom: BTreeMap<String, String> =
        serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
    let mut table = builtin(language);
    table.extend(custom);
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogs_have_the_same_keys_and_placeholders() {
        let ja = builtin("ja");
        let en = builtin("en");
        assert_eq!(ja.keys().collect::<Vec<_>>(), en.keys().collect::<Vec<_>>());
        let pattern = regex::Regex::new(r"\{([A-Za-z0-9_]+)\}").unwrap();
        for (key, text) in ja {
            let fields = |s: &str| {
                pattern
                    .captures_iter(s)
                    .map(|c| c[1].to_string())
                    .collect::<std::collections::BTreeSet<_>>()
            };
            assert_eq!(fields(&text), fields(&en[&key]), "{key}");
        }
    }
    #[test]
    fn interpolation_does_not_reinterpret_user_data() {
        assert_eq!(
            render(
                "{count}: {text}",
                &[("count", "2".into()), ("text", "{count}".into())]
            ),
            "2: {count}"
        );
    }
}

#[cfg(test)]
mod override_tests {
    use super::*;
    #[test]
    fn users_can_override_a_translation_without_replacing_the_whole_catalog() {
        let table = merge_overrides("en", r#"{"app.211":"Store my map"}"#).unwrap();
        assert_eq!(table["app.211"], "Store my map");
        assert_eq!(table["app.212"], "Open map");
        assert!(merge_overrides("en", "{bad").is_err());
    }
}
