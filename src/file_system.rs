use std::{cell::RefCell, path::Path};

pub struct FileSystem {}

pub trait FileOperations {
    fn copy_file(&self, from: &Path, to: &Path);
}

#[cfg(test)]
#[derive(Default)]
pub struct FileSystemSpy {
    pub files_copied: RefCell<Vec<String>>,
}

#[cfg(test)]
impl FileOperations for FileSystemSpy {
    fn copy_file(&self, from: &Path, _to: &Path) {
        self.files_copied
            .borrow_mut()
            .push(from.to_string_lossy().to_string());
    }
}
