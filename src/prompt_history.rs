//! UTF-8 JSON is deliberately user-editable; newest exact instruction is first.
use anyhow::{Context, Result};
use std::path::Path;

pub fn load(path: &Path) -> Result<Vec<String>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let items: Vec<String> =
                serde_json::from_slice(&bytes).context(crate::i18n::tr("history.invalid"))?;
            let mut result = Vec::new();
            for text in items {
                if !text.trim().is_empty() && !result.contains(&text) && result.len() < 10 {
                    result.push(text);
                }
            }
            Ok(result)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}
pub fn record(path: &Path, text: &str) -> Result<Vec<String>> {
    let mut items = load(path)?;
    if !text.trim().is_empty() {
        items.retain(|s| s != text);
        items.insert(0, text.to_owned());
        items.truncate(10);
        crate::storage::atomic_write(path, &serde_json::to_vec_pretty(&items)?)?;
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recency_exact_match_eviction_and_restart() {
        std::fs::create_dir_all("artifacts/tmp").unwrap();
        let dir = tempfile::tempdir_in("artifacts/tmp").unwrap();
        let p = dir.path().join("history.json");
        for n in 0..12 {
            record(&p, &format!("指示{n}")).unwrap();
        }
        let before = load(&p).unwrap();
        assert_eq!(before.len(), 10);
        assert_eq!(before[0], "指示11");
        assert_eq!(before[9], "指示2");
        let items = record(&p, "指示5").unwrap();
        assert_eq!(items[0], "指示5");
        assert_eq!(items.len(), 10);
        let items = record(&p, " 指示5 ").unwrap();
        assert_eq!(&items[..2], &[" 指示5 ", "指示5"]);
        assert_eq!(load(&p).unwrap(), items);
        assert_eq!(record(&p, "\t ").unwrap(), items);
    }
    #[test]
    fn malformed_history_is_not_overwritten() {
        std::fs::create_dir_all("artifacts/tmp").unwrap();
        let dir = tempfile::tempdir_in("artifacts/tmp").unwrap();
        let p = dir.path().join("history.json");
        std::fs::write(&p, "invalid").unwrap();
        assert!(record(&p, "new").is_err());
        assert_eq!(std::fs::read_to_string(p).unwrap(), "invalid");
    }
}
