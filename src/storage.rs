use crate::i18n::{msg, tr};
use crate::{mm, model::Document};
use anyhow::{Context, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
            };
            let a: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
            let b: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            if unsafe {
                MoveFileExW(
                    a.as_ptr(),
                    b.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        #[cfg(not(windows))]
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result.with_context(|| msg("storage.248", &[("0", format!("{}", path.display()))]))
}
pub fn save(path: &Path, doc: &Document) -> Result<()> {
    let bytes = mm::write(doc)?;
    if path.exists() {
        let folder = path
            .parent()
            .unwrap_or(Path::new("."))
            .join(".mindglider-backup");
        fs::create_dir_all(&folder)?;
        let name = path
            .file_name()
            .context(tr("storage.249"))?
            .to_string_lossy();
        for i in (1..10).rev() {
            let src = folder.join(format!("{name}.{i}.mm"));
            let dst = folder.join(format!("{name}.{}.mm", i + 1));
            if src.exists() {
                let b = fs::read(src)?;
                atomic_write(&dst, &b)?;
            }
        }
        atomic_write(&folder.join(format!("{name}.1.mm")), &fs::read(path)?)?;
    }
    atomic_write(path, &bytes)
}
pub fn data_dir() -> PathBuf {
    if let Ok(path) =
        std::env::var("MINDGLIDER_DATA_DIR").or_else(|_| std::env::var(crate::compat::OLD_DATA_ENV))
    {
        return PathBuf::from(path);
    }
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent().unwrap_or(Path::new(".")).join("data")
}
pub fn resolve(base: Option<&Path>, uri: &str) -> PathBuf {
    if let Ok(url) = url::Url::parse(uri)
        && url.scheme() == "file"
        && let Ok(p) = url.to_file_path()
    {
        return p;
    }
    let path = PathBuf::from(uri.replace("%20", " "));
    if path.is_absolute() {
        path
    } else {
        base.and_then(Path::parent)
            .unwrap_or(Path::new("."))
            .join(path)
    }
}
