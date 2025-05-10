mod file_system;

use std::path::PathBuf;

use file_system::FileSystem;

fn main() -> Result<(), anyhow::Error> {
    Ok(())
}

trait CrateDirectory {
    fn get_license(&self) -> Option<PathBuf>;
}

fn find_and_copy_licenses<C: CrateDirectory>(
    crate_directories: Vec<C>,
    filesystem: &impl FileSystem,
) {
    for crate_directory in crate_directories {
        if let Some(license_path) = crate_directory.get_license() {
            filesystem.copy_file(&license_path, &PathBuf::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::find_and_copy_licenses;
    use crate::{CrateDirectory, file_system::FileSystemSpy};

    #[test]
    fn when_there_are_no_crates_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();

        find_and_copy_licenses(Vec::<CrateDirectoryFake>::new(), &file_system_spy);

        assert!(file_system_spy.files_copied.take().is_empty())
    }

    struct CrateDirectoryFake {
        license: Option<String>,
    }

    impl CrateDirectoryFake {
        fn containing_license(license_name: Option<&str>) -> Self {
            Self {
                license: license_name.map(|license_name| license_name.to_string()),
            }
        }
    }

    impl CrateDirectory for CrateDirectoryFake {
        fn get_license(&self) -> Option<PathBuf> {
            self.license.as_ref().map(|license| PathBuf::from(license))
        }
    }

    #[test]
    fn when_there_is_one_crate_with_no_licenses_then_no_license_files_are_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::containing_license(None);

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert!(file_system_spy.files_copied.take().is_empty())
    }

    #[test]
    fn when_there_is_one_crate_with_one_license_then_one_license_file_copied() {
        let file_system_spy = FileSystemSpy::default();
        let crate_directory_fake = CrateDirectoryFake::containing_license(Some("LICENSE-MIT"));

        find_and_copy_licenses(vec![crate_directory_fake], &file_system_spy);

        assert_eq!(vec!["LICENSE-MIT"], file_system_spy.files_copied.take())
    }
}
