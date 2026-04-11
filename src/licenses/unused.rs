use crate::cargo_metadata::Package;
use crate::config::CrateConfig;
use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::is_license::is_license;
use crate::licenses::status::LicenseStatus;
use crate::licenses::validate::validate_licenses;
use crate::log::warning;
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct UnusedConfigs(Vec<(String, UnusedConfigReason)>);

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum UnusedConfigReason {
    CrateNotFound,
    AllowNotRequired,
    SkipNotRequired(Vec<String>),
}

impl UnusedConfigs {
    pub fn any(&self) -> bool {
        !self.0.is_empty()
    }
}

impl Display for UnusedConfigs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        writeln!(
            f,
            "{} - entries in the config are not being used:",
            warning(&format!("{}", "unused".bold())),
        )?;
        for (crate_name, reason) in &self.0 {
            write!(f, "\t{}", crate_name.bold())?;
            match reason {
                UnusedConfigReason::CrateNotFound => {
                    writeln!(f, " - crate not found in dependencies")?;
                }
                UnusedConfigReason::AllowNotRequired => {
                    writeln!(f, " - 'allow' is not required")?;
                }
                UnusedConfigReason::SkipNotRequired(files) => {
                    writeln!(
                        f,
                        " - 'skip' for {} is not required",
                        files.iter().sorted().join(", ")
                    )?;
                }
            }
        }
        Ok(())
    }
}

pub fn find_unused_configs(
    file_io: &impl FileIO,
    all_licenses: &HashMap<Package, Vec<DirEntry>>,
    crate_configs: &HashMap<String, CrateConfig>,
) -> UnusedConfigs {
    let package_map: HashMap<&str, (&Package, &Vec<DirEntry>)> = all_licenses
        .iter()
        .map(|(package, licenses)| (package.normalised_name.as_str(), (package, licenses)))
        .collect();

    let unused = crate_configs
        .iter()
        .sorted_by_key(|(name, _)| (*name).clone())
        .flat_map(|(crate_name, config)| {
            find_unused_for_crate(
                file_io,
                crate_name,
                config,
                package_map.get(crate_name.as_str()),
            )
        })
        .collect();

    UnusedConfigs(unused)
}

fn find_unused_for_crate(
    file_io: &impl FileIO,
    crate_name: &str,
    config: &CrateConfig,
    package_entry: Option<&(&Package, &Vec<DirEntry>)>,
) -> Vec<(String, UnusedConfigReason)> {
    let Some((package, licenses)) = package_entry else {
        return vec![(crate_name.to_string(), UnusedConfigReason::CrateNotFound)];
    };

    let mut unused = Vec::new();

    if let Some(reason) = check_unused_allow(file_io, config, package, licenses) {
        unused.push((crate_name.to_string(), reason));
    }

    if let Some(reason) = check_unused_skip(file_io, config, package) {
        unused.push((crate_name.to_string(), reason));
    }

    unused
}

fn check_unused_allow(
    file_io: &impl FileIO,
    config: &CrateConfig,
    package: &Package,
    licenses: &[DirEntry],
) -> Option<UnusedConfigReason> {
    config.allow.as_ref().and_then(|_| {
        let raw_status = validate_licenses(
            file_io,
            package.license.as_deref().map(License::parse).as_ref(),
            licenses,
        );
        (raw_status == LicenseStatus::Valid).then_some(UnusedConfigReason::AllowNotRequired)
    })
}

fn check_unused_skip(
    file_io: &impl FileIO,
    config: &CrateConfig,
    package: &Package,
) -> Option<UnusedConfigReason> {
    if config.skip.is_empty() {
        return None;
    }

    let unused_skips = find_unused_skip_files(file_io, package, &config.skip);
    (!unused_skips.is_empty()).then_some(UnusedConfigReason::SkipNotRequired(unused_skips))
}

fn find_unused_skip_files(
    file_io: &impl FileIO,
    package: &Package,
    skip: &[String],
) -> Vec<String> {
    let Ok(dir_entries) = file_io.read_dir(package.path.as_ref()) else {
        return Vec::new();
    };

    let license_files: HashSet<String> = dir_entries
        .iter()
        .filter(|entry| is_license(entry))
        .filter_map(|entry| entry.name.to_str().map(|name| name.to_string()))
        .collect();

    skip.iter()
        .filter(|file| !license_files.contains(file.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::config::CrateConfig;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::licenses::status::LicenseStatus;
    use crate::licenses::unused::{UnusedConfigReason, UnusedConfigs, find_unused_configs};
    use crate::licenses::validate::LICENSE_TEXTS;
    use cargo_metadata::camino::Utf8PathBuf;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn no_unused_configs_when_no_config() {
        let file_io_spy = FileIOSpy::default();
        let unused = find_unused_configs(&file_io_spy, &HashMap::new(), &HashMap::new());
        assert!(!unused.any());
    }

    #[test]
    fn unused_config_when_crate_not_in_dependencies() {
        let file_io_spy = FileIOSpy::default();
        let crate_configs = std::iter::once((
            "missing_crate".to_string(),
            CrateConfig {
                skip: vec![],
                allow: Some(LicenseStatus::TooFew),
                include: vec![],
            },
        ))
        .collect();

        let unused = find_unused_configs(&file_io_spy, &HashMap::new(), &crate_configs);
        assert_eq!(
            unused.0,
            vec![(
                "missing_crate".to_string(),
                UnusedConfigReason::CrateNotFound
            )]
        );
    }

    #[test]
    fn unused_allow_when_status_already_valid() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([Ok(license_text("MIT"))]);

        let package = Package {
            normalised_name: "some_crate".to_string(),
            path: Utf8PathBuf::default(),
            url: None,
            license: Some("MIT".to_string()),
        };

        let all_licenses: HashMap<_, _> = std::iter::once((
            package,
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: true,
            }],
        ))
        .collect();

        let crate_configs = std::iter::once((
            "some_crate".to_string(),
            CrateConfig {
                skip: vec![],
                allow: Some(LicenseStatus::TooFew),
                include: vec![],
            },
        ))
        .collect();

        let unused = find_unused_configs(&file_io_spy, &all_licenses, &crate_configs);
        assert_eq!(
            unused.0,
            vec![(
                "some_crate".to_string(),
                UnusedConfigReason::AllowNotRequired
            )]
        );
    }

    #[test]
    fn no_unused_when_allow_is_needed() {
        let file_io_spy = FileIOSpy::default();

        let package = Package {
            normalised_name: "some_crate".to_string(),
            path: Utf8PathBuf::default(),
            url: None,
            license: Some("MIT".to_string()),
        };

        let all_licenses: HashMap<_, _> = std::iter::once((package, vec![])).collect();

        let crate_configs = std::iter::once((
            "some_crate".to_string(),
            CrateConfig {
                skip: vec![],
                allow: Some(LicenseStatus::Empty),
                include: vec![],
            },
        ))
        .collect();

        let unused = find_unused_configs(&file_io_spy, &all_licenses, &crate_configs);
        assert!(!unused.any());
    }

    #[test]
    fn unused_skip_when_files_not_present() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("LICENSE"),
            path: PathBuf::new(),
            is_file: true,
        }])]);

        let all_licenses: HashMap<_, _> = std::iter::once((
            Package::called("some_crate"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::new(),
                is_file: true,
            }],
        ))
        .collect();

        let crate_configs = std::iter::once((
            "some_crate".to_string(),
            CrateConfig {
                skip: vec!["COPYING".to_string()],
                allow: None,
                include: vec![],
            },
        ))
        .collect();

        let unused = find_unused_configs(&file_io_spy, &all_licenses, &crate_configs);
        assert_eq!(
            unused.0,
            vec![(
                "some_crate".to_string(),
                UnusedConfigReason::SkipNotRequired(vec!["COPYING".to_string()])
            )]
        );
    }

    #[test]
    fn no_unused_when_skip_files_exist() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![
            DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::new(),
                is_file: true,
            },
            DirEntry {
                name: OsString::from("COPYING"),
                path: PathBuf::new(),
                is_file: true,
            },
        ])]);

        let all_licenses: HashMap<_, _> = std::iter::once((
            Package::called("some_crate"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::new(),
                is_file: true,
            }],
        ))
        .collect();

        let crate_configs = std::iter::once((
            "some_crate".to_string(),
            CrateConfig {
                skip: vec!["COPYING".to_string()],
                allow: None,
                include: vec![],
            },
        ))
        .collect();

        let unused = find_unused_configs(&file_io_spy, &all_licenses, &crate_configs);
        assert!(!unused.any());
    }

    #[test]
    fn display_unused_configs() {
        let unused = UnusedConfigs(vec![
            (
                "another_crate".to_string(),
                UnusedConfigReason::AllowNotRequired,
            ),
            (
                "missing_crate".to_string(),
                UnusedConfigReason::CrateNotFound,
            ),
            (
                "third_crate".to_string(),
                UnusedConfigReason::SkipNotRequired(vec!["COPYING".to_string()]),
            ),
        ]);

        assert_eq!(
            "warning: unused - entries in the config are not being used:\n\
             another_crate - 'allow' is not required\n\
             missing_crate - crate not found in dependencies\n\
             third_crate - 'skip' for COPYING is not required\n",
            strip_ansi_escapes::strip_str(unused.to_string())
        );
    }

    #[test]
    fn display_empty_unused_configs() {
        let unused = UnusedConfigs(Vec::new());
        assert!(unused.to_string().is_empty());
    }

    fn license_text(id: &str) -> String {
        LICENSE_TEXTS.get(id).unwrap().to_string()
    }
}
