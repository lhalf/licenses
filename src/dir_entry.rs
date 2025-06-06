use std::path::PathBuf;

pub trait DirEntry {
    fn filename(&self) -> String;
    fn path(&self) -> PathBuf;
}

impl DirEntry for std::fs::DirEntry {
    fn filename(&self) -> String {
        // currently ignore filename containing invalid utf8
        self.file_name().into_string().unwrap_or_else(|_| String::new())
    }

    fn path(&self) -> PathBuf {
        self.path()
    }
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