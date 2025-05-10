mod file_system;

use file_system::FileSystem;

fn main() -> Result<(), anyhow::Error> {
    Ok(())
}

trait CrateDirectory {}

fn find_and_copy_licenses<C: CrateDirectory>(
    _crate_directories: Vec<C>,
    _filesystem: &impl FileSystem,
) {
}

#[cfg(test)]
mod tests {
    use super::find_and_copy_licenses;
    use crate::{CrateDirectory, file_system::FileSystemSpy};

    #[test]
    fn when_there_are_no_crates_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();

        find_and_copy_licenses(Vec::<CrateDirectoryFake>::new(), &file_system_spy);

        assert!(file_system_spy.files_copied.is_empty())
    }

    #[derive(Default)]
    struct CrateDirectoryFake {}

    impl CrateDirectory for CrateDirectoryFake {}

    #[test]
    fn when_there_is_one_crate_with_no_licenses_then_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::default();

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert!(file_system_spy.files_copied.is_empty())
    }
}
