use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use crate::log::{Log, LogLevel};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn diff_licenses(
    file_io: &impl FileIO,
    logger: &impl Log,
    path: PathBuf,
    found_licenses: HashMap<Package, Vec<DirEntry>>,
) -> anyhow::Result<HashSet<String>> {
    let current_licenses = set_of_current_licenses(file_io.read_dir(&path)?);
    let found_licenses = flatten(found_licenses);
    let diff = found_licenses
        .difference(&current_licenses)
        .map(|license| {
            logger.log(
                LogLevel::Warning,
                &format!("found license not in output folder: {}", license.bold()),
            );
            license.to_owned()
        })
        .collect();
    Ok(diff)
}

fn set_of_current_licenses(dir_entries: Vec<DirEntry>) -> HashSet<String> {
    dir_entries
        .into_iter()
        .filter(|dir_entry| dir_entry.is_file)
        .map(|dir_entry| dir_entry.name.to_string_lossy().into_owned())
        .collect()
}

fn flatten(found_licenses: HashMap<Package, Vec<DirEntry>>) -> HashSet<String> {
    found_licenses
        .into_iter()
        .flat_map(|(package, dir_entries)| {
            dir_entries.into_iter().map(move |dir_entry| {
                format!(
                    "{}-{}",
                    package.normalised_name,
                    dir_entry.name.to_string_lossy()
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::diff_licenses;
    use crate::cargo_metadata::Package;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::log::LogSpy;
    use std::collections::{HashMap, HashSet};
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn failure_to_read_licenses_directory_causes_error_and_no_logs() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        assert!(
            diff_licenses(
                &file_io_spy,
                &LogSpy::default(),
                PathBuf::new(),
                HashMap::new()
            )
            .is_err()
        );
    }

    #[test]
    fn no_differences_in_licenses_causes_no_error_or_logs() {
        let file_io_spy = FileIOSpy::default();

        let current_dir_entries = vec![DirEntry {
            name: OsString::from("example-LICENSE"),
            path: Default::default(),
            is_file: true,
        }];

        file_io_spy.read_dir.returns.set([Ok(current_dir_entries)]);

        let found_licenses = [(
            Package::called("example"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        assert!(
            diff_licenses(
                &file_io_spy,
                &LogSpy::default(),
                PathBuf::new(),
                found_licenses
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn does_not_read_directories_in_current_licenses_folder() {
        let file_io_spy = FileIOSpy::default();
        let current_dir_entries = vec![
            DirEntry {
                name: OsString::from("example-LICENSE"),
                path: Default::default(),
                is_file: true,
            },
            DirEntry {
                name: OsString::from("dir-not-a-file"),
                path: Default::default(),
                is_file: false,
            },
        ];
        file_io_spy.read_dir.returns.set([Ok(current_dir_entries)]);

        let found_licenses = [(
            Package::called("example"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        assert!(
            diff_licenses(
                &file_io_spy,
                &LogSpy::default(),
                PathBuf::new(),
                found_licenses
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn differences_in_licenses_causes_error_and_logs() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(Vec::new())]);

        let log_spy = LogSpy::default();
        log_spy.log.returns.set([()]);

        let found_licenses = [(
            Package::called("example"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        let expected_diff = HashSet::from(["example-LICENSE".to_string()]);

        assert_eq!(
            expected_diff,
            diff_licenses(&file_io_spy, &log_spy, PathBuf::new(), found_licenses).unwrap()
        );
    }
}
