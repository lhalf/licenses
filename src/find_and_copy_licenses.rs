use std::path::PathBuf;
use crate::file_system::FileOperations;
use crate::find_licenses::FindLicenses;

fn find_and_copy_licenses<F: FindLicenses>(
    crate_directories: Vec<F>,
    filesystem: &impl FileOperations,
) {
    crate_directories
        .into_iter()
        .map(|crate_directory| crate_directory.find_licenses().unwrap_or(vec![]))
        .flatten()
        .for_each(|license_path| {
            filesystem.copy_file(&license_path, &PathBuf::new());
        });
}

#[cfg(test)]
mod tests {
    use super::find_and_copy_licenses;
    use crate::{file_system::FileSystemSpy, find_licenses::CrateDirectoryFake};

    #[test]
    fn when_there_are_no_crates_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();

        find_and_copy_licenses(Vec::<CrateDirectoryFake>::new(), &file_system_spy);

        assert!(file_system_spy.files_copied.take().is_empty())
    }

    #[test]
    fn when_there_is_one_crate_and_finding_licenses_fails_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::failing();

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert!(file_system_spy.files_copied.take().is_empty())
    }

    #[test]
    fn when_there_is_one_crate_with_no_licenses_then_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::containing_licenses(vec![]);

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert!(file_system_spy.files_copied.take().is_empty())
    }

    #[test]
    fn when_there_is_one_crate_with_one_license_then_one_license_file_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::containing_licenses(vec!["LICENSE-MIT"]);

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert_eq!(vec!["LICENSE-MIT"], file_system_spy.files_copied.take())
    }

    #[test]
    fn when_there_is_two_crates_one_with_license_one_without_then_one_license_file_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_1_directory_fake = CrateDirectoryFake::containing_licenses(vec!["LICENSE-MIT"]);
        let crate_2_directory_fake = CrateDirectoryFake::containing_licenses(vec![]);

        find_and_copy_licenses(
            vec![crate_1_directory_fake, crate_2_directory_fake],
            &file_system_spy,
        );

        assert_eq!(vec!["LICENSE-MIT"], file_system_spy.files_copied.take())
    }

    #[test]
    fn when_there_is_one_crate_with_multiple_licenses_then_multiple_license_files_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake =
            CrateDirectoryFake::containing_licenses(vec!["LICENSE-MIT", "LICENSE-APACHE"]);

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert_eq!(
            vec!["LICENSE-MIT", "LICENSE-APACHE"],
            file_system_spy.files_copied.take()
        )
    }
}
