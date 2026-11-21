use anyhow::{Context, Result, ensure};
use eframe::egui;
use mindglider::i18n::tr;
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime},
};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    length: u64,
    modified: SystemTime,
    #[cfg(windows)]
    identity: (u32, u64),
    #[cfg(unix)]
    identity: (u64, u64),
}

impl FileStamp {
    fn read(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        Self::from_file(&file)
    }

    fn from_file(file: &File) -> Result<Self> {
        let metadata = file.metadata()?;
        ensure!(metadata.is_file(), "{}", tr("update.file_unavailable"));
        #[cfg(windows)]
        let identity = {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::{
                BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
            };
            let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
            if unsafe { GetFileInformationByHandle(file.as_raw_handle().cast(), info.as_mut_ptr()) }
                == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            let info = unsafe { info.assume_init() };
            (
                info.dwVolumeSerialNumber,
                (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
            )
        };
        #[cfg(unix)]
        let identity = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };
        Ok(Self {
            length: metadata.len(),
            modified: metadata.modified()?,
            #[cfg(any(windows, unix))]
            identity,
        })
    }
}

pub struct Watch {
    path: PathBuf,
    running: FileStamp,
    next_check: Instant,
    offered: Option<FileStamp>,
    deferred: Option<FileStamp>,
}

impl Watch {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            running: FileStamp::read(&path)?,
            path,
            next_check: Instant::now() + POLL_INTERVAL,
            offered: None,
            deferred: None,
        })
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        if now >= self.next_check {
            self.refresh();
            self.next_check = now + POLL_INTERVAL;
        }
        ctx.request_repaint_after(self.next_check.saturating_duration_since(now));
    }

    pub(crate) fn refresh(&mut self) {
        let found = FileStamp::read(&self.path).ok();
        let offered = found.filter(|stamp| *stamp != self.running && Some(*stamp) != self.deferred);
        if offered.is_some() && offered != self.offered {
            log::info!("UPDATE_DETECTED {}", self.path.display());
        }
        self.offered = offered;
    }

    pub fn available(&self) -> bool {
        self.offered.is_some()
    }

    pub fn defer(&mut self) {
        self.deferred = self.offered.take();
    }

    pub fn approve(&mut self) -> Result<Restart> {
        let previous = self.offered;
        self.refresh();
        let stamp = self
            .offered
            .filter(|stamp| Some(*stamp) == previous)
            .context(tr("update.file_changed"))?;
        Ok(Restart {
            path: self.path.clone(),
            stamp,
        })
    }
}

pub struct Restart {
    path: PathBuf,
    stamp: FileStamp,
}

impl Restart {
    pub fn launch(self) -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        // Keep the approved file from being replaced until Windows has opened it.
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
            options.share_mode(FILE_SHARE_READ);
        }
        let file = options
            .open(&self.path)
            .context(tr("update.file_changed"))?;
        ensure!(
            FileStamp::from_file(&file)? == self.stamp,
            "{}",
            tr("update.file_changed")
        );
        let child = Command::new(&self.path).spawn()?;
        drop(file);
        log::info!("UPDATE_RESTARTED pid={}", child.id());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn replacement_is_detected_and_later_suppresses_only_that_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mindglider.exe");
        fs::write(&path, "original").unwrap();
        let mut watch = Watch::new(path.clone()).unwrap();
        watch.refresh();
        assert!(!watch.available());
        fs::rename(&path, dir.path().join("original.exe")).unwrap();
        watch.refresh();
        assert!(!watch.available());
        fs::write(&path, "updated").unwrap();
        watch.refresh();
        assert!(watch.available());
        watch.defer();
        watch.refresh();
        assert!(!watch.available());
        fs::rename(&path, dir.path().join("second.exe")).unwrap();
        fs::write(&path, "another update").unwrap();
        watch.refresh();
        assert!(watch.available());
        assert!(watch.approve().is_ok());
    }

    #[test]
    fn approval_does_not_launch_a_different_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mindglider.exe");
        fs::write(&path, "original").unwrap();
        let mut watch = Watch::new(path.clone()).unwrap();
        fs::write(&path, "first update").unwrap();
        watch.refresh();
        fs::write(&path, "second update").unwrap();
        assert!(watch.approve().is_err());
        let approved = watch.approve().unwrap();
        fs::remove_file(path).unwrap();
        assert!(approved.launch().is_err());
    }
}
