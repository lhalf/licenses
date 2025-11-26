use crate::cargo_metadata::Package;
use crate::config::{CrateConfig, IncludedLicense};
use crate::file_io::{DirEntry, FileIO};
use crate::log::{LogLevel, log_message};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct LicenseDiff {
    additional: HashSet<String>,
    missing: HashSet<String>,
}

impl Display for LicenseDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.additional.is_empty() {
            writeln!(
                f,
                "{}",
                log_message(
                    LogLevel::Warning,
                    &format!(
                        "found additional licenses in the output folder\n   {}",
                        self.additional.iter().join("\n ")
                    )
                )
            )?;
        }
        if !self.missing.is_empty() {
            writeln!(
                f,
                "{}",
                log_message(
                    LogLevel::Warning,
                    &format!(
                        "found licenses missing from the output folder\n    {}",
                        self.missing.iter().join("\n    ")
                    )
                )
            )?;
        }
        Ok(())
    }
}

impl LicenseDiff {
    pub fn is_empty(&self) -> bool {
        self.additional.is_empty() && self.missing.is_empty()
    }
}

pub fn diff_licenses(
    file_io: &impl FileIO,
    path: PathBuf,
    crate_configs: &HashMap<String, CrateConfig>,
    found_licenses: HashMap<Package, Vec<DirEntry>>,
) -> anyhow::Result<LicenseDiff> {
    let current_licenses = set_of_current_licenses(file_io.read_dir(&path)?);
    let mut found_licenses = flatten(found_licenses);
    found_licenses.extend(included_licenses(crate_configs));

    Ok(LicenseDiff {
        additional: current_licenses
            .difference(&found_licenses)
            .cloned()
            .collect(),
        missing: found_licenses
            .difference(&current_licenses)
            .cloned()
            .collect(),
    })
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

fn included_licenses(crate_configs: &HashMap<String, CrateConfig>) -> HashSet<String> {
    crate_configs
        .iter()
        .flat_map(|(crate_name, config)| {
            config.include.iter().map(move |license| {
                format!(
                    "{crate_name}-{}",
                    match license {
                        IncludedLicense::Text { name, .. } => name,
                    }
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{LicenseDiff, diff_licenses};
    use crate::cargo_metadata::Package;
    use crate::config::{CrateConfig, IncludedLicense};
    use crate::file_io::{DirEntry, FileIOSpy};
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
                PathBuf::new(),
                &HashMap::new(),
                HashMap::new()
            )
            .is_err()
        );
    }

    #[test]
    fn no_differences_in_licenses_causes_no_error() {
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
                PathBuf::new(),
                &HashMap::new(),
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
                PathBuf::new(),
                &HashMap::new(),
                found_licenses
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn missing_licenses() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(Vec::new())]);

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

        let expected_diff = LicenseDiff {
            additional: HashSet::new(),
            missing: HashSet::from(["example-LICENSE".to_string()]),
        };

        assert_eq!(
            expected_diff,
            diff_licenses(
                &file_io_spy,
                PathBuf::new(),
                &HashMap::new(),
                found_licenses
            )
            .unwrap()
        );
    }

    #[test]
    fn additional_licenses() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("example-LICENSE"),
            path: Default::default(),
            is_file: true,
        }])]);

        let found_licenses = HashMap::new();

        let expected_diff = LicenseDiff {
            additional: HashSet::from(["example-LICENSE".to_string()]),
            missing: HashSet::new(),
        };

        assert_eq!(
            expected_diff,
            diff_licenses(
                &file_io_spy,
                PathBuf::new(),
                &HashMap::new(),
                found_licenses
            )
            .unwrap()
        );
    }

    #[test]
    fn additional_licenses_are_okay_if_they_are_from_included_section_in_config() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("example-LICENSE"),
            path: Default::default(),
            is_file: true,
        }])]);

        let found_licenses = HashMap::new();

        let config = [(
            "example".to_string(),
            CrateConfig {
                skip: vec![],
                allow: None,
                include: vec![IncludedLicense::Text {
                    name: "LICENSE".to_string(),
                    text: "I got included!".to_string(),
                }],
            },
        )]
        .into_iter()
        .collect();

        // there is a diff if we don't use the config
        assert!(
            !diff_licenses(
                &file_io_spy,
                PathBuf::new(),
                &HashMap::new(),
                found_licenses.clone()
            )
            .unwrap()
            .is_empty()
        );

        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("example-LICENSE"),
            path: Default::default(),
            is_file: true,
        }])]);

        // no diff if the additional license is from the included section of the config
        assert!(
            diff_licenses(&file_io_spy, PathBuf::new(), &config, found_licenses)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn empty_diff_produces_no_output() {
        assert_eq!(
            "",
            LicenseDiff {
                additional: HashSet::new(),
                missing: HashSet::new(),
            }
            .to_string()
        );
    }
}
