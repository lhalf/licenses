#![allow(unused)]

use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn diff_licenses(
    path: PathBuf,
    file_io: &impl FileIO,
    found_licenses: HashMap<Package, Vec<DirEntry>>,
) -> anyhow::Result<HashSet<String>> {
    let current_licenses = set_of_current_licenses(file_io.read_dir(&path)?);
    let found_licenses = flatten(found_licenses);
    Ok(current_licenses
        .difference(&found_licenses)
        .cloned()
        .collect())
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
    use std::collections::{HashMap, HashSet};
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn failure_to_read_licenses_directory_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        assert!(diff_licenses(PathBuf::new(), &file_io_spy, HashMap::new()).is_err());
    }

    #[test]
    fn no_differences_in_licenses() {
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
            diff_licenses(PathBuf::new(), &file_io_spy, found_licenses)
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
            diff_licenses(PathBuf::new(), &file_io_spy, found_licenses)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn differences_in_licenses() {
        let file_io_spy = FileIOSpy::default();
        let current_dir_entries = vec![
            DirEntry {
                name: OsString::from("example-LICENSE"),
                path: Default::default(),
                is_file: true,
            },
            DirEntry {
                name: OsString::from("other-LICENSE-APACHE"),
                path: Default::default(),
                is_file: true,
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

        let expected_diff = HashSet::from(["other-LICENSE-APACHE".to_string()]);

        assert_eq!(
            expected_diff,
            diff_licenses(PathBuf::new(), &file_io_spy, found_licenses).unwrap()
        );
    }
}
