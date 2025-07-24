use anyhow::Context;
use std::path::Path;

pub struct FileSystem {}

#[cfg_attr(test, autospy::autospy)]
pub trait FileIO {
    fn copy_file(&self, from: &Path, to: &Path) -> anyhow::Result<()>;
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
}
