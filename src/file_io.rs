use anyhow::Context;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub struct FileSystem {}

#[cfg_attr(test, autospy::autospy)]
pub trait FileIO {
    fn copy_file(&self, from: &Path, to: &Path) -> anyhow::Result<()>;
    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<DirEntry>>;
}

impl FileIO for FileSystem {
    fn copy_file(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        std::fs::copy(from, to).context(format!(
            "failed to copy {} to {}",
            from.display(),
            to.display()
        ))?;
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<DirEntry>> {
        std::fs::read_dir(path)
            .context("failed to read directory")?
            .map(DirEntry::try_from)
            .collect()
    }
}

pub struct DirEntry {
    pub name: OsString,
    pub path: PathBuf,
    pub is_file: bool,
}

impl DirEntry {
    fn try_from(dir_entry: std::io::Result<std::fs::DirEntry>) -> anyhow::Result<Self> {
        let dir_entry = dir_entry.context("invalid dir entry")?;
        Ok(Self {
            name: dir_entry.file_name(),
            path: dir_entry.path(),
            is_file: dir_entry.path().is_file(),
        })
    }
}
