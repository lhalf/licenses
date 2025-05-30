use std::path::PathBuf;

pub trait DirEntry {
    fn filename(&self) -> String;
    fn path(&self) -> PathBuf;
}

#[cfg(test)]
pub struct DirEntryFake {
    pub filename: &'static str,
    pub path: PathBuf,
}

#[cfg(test)]
impl DirEntry for DirEntryFake {
    fn filename(&self) -> String {
        self.filename.to_string()
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}