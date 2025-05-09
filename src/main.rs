mod file_system;

use file_system::FileSystem;

fn main() -> Result<(), anyhow::Error> {
    Ok(())
}

fn find_and_copy_licenses(_crate_directories: Vec<()>, _filesystem: &impl FileSystem) {}

#[cfg(test)]
mod tests {
    use super::{FileSystem, find_and_copy_licenses};

    #[derive(Default)]
    struct FileSystemSpy {
        files_copied: Vec<()>,
    }

    impl FileSystem for FileSystemSpy {}

    #[test]
    fn when_there_are_no_crates_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();

        find_and_copy_licenses(Vec::new(), &file_system_spy);

        assert!(file_system_spy.files_copied.is_empty())
    }
}
