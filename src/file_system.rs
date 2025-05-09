pub trait FileSystem {}

#[cfg(test)]
#[derive(Default)]
pub struct FileSystemSpy {
    pub files_copied: Vec<()>,
}

#[cfg(test)]
impl FileSystem for FileSystemSpy {}
