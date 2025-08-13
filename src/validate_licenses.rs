use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::license::License;
use crate::{note, warn};
use colored::Colorize;

#[derive(PartialEq, Debug)]
pub enum LicenseStatus {
    Valid,
    Empty,
    NoneDeclared,
    Mismatch,
}

impl LicenseStatus {
    pub fn warn(&self, package: &Package) {
        match self {
            LicenseStatus::Valid => {}
            LicenseStatus::Empty => {
                warn!(
                    "did not find any licenses for {} - {}",
                    package.normalised_name.bold(),
                    match &package.url {
                        Some(url) => format!("try looking here: {url}"),
                        None => "no url".to_string(),
                    }
                );
            }
            LicenseStatus::NoneDeclared => {
                note!(
                    "no declared licenses for {}",
                    package.normalised_name.bold()
                );
            }
            LicenseStatus::Mismatch => {
                note!(
                    "declared licenses did not match found licenses for {}",
                    package.normalised_name.bold()
                );
            }
        }
    }
}

pub fn validate_licenses(
    declared_licenses: &Option<License>,
    actual_licenses: &[DirEntry],
) -> LicenseStatus {
    if actual_licenses.is_empty() {
        return LicenseStatus::Empty;
    }

    match declared_licenses {
        None => LicenseStatus::NoneDeclared,
        Some(licenses) if licenses.requirements().count() != actual_licenses.len() => {
            LicenseStatus::Mismatch
        }
        _ => LicenseStatus::Valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn no_licenses_found() {
        assert_eq!(
            LicenseStatus::Empty,
            validate_licenses(&Some(License::parse("MIT")), &[])
        );
    }

    #[test]
    fn no_listed_license() {
        assert_eq!(
            LicenseStatus::NoneDeclared,
            validate_licenses(
                &None,
                &[DirEntry {
                    name: Default::default(),
                    path: Default::default(),
                    is_file: false,
                }]
            )
        );
    }

    #[test]
    fn mismatch_too_few_licenses() {
        assert_eq!(
            LicenseStatus::Mismatch,
            validate_licenses(
                &Some(License::parse("MIT OR Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: Default::default(),
                    is_file: true,
                }]
            )
        );
        assert_eq!(
            LicenseStatus::Mismatch,
            validate_licenses(
                &Some(License::parse("MIT/Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: Default::default(),
                    is_file: true,
                }]
            )
        );
        assert_eq!(
            LicenseStatus::Mismatch,
            validate_licenses(
                &Some(License::parse("(MIT OR Apache-2.0) AND Unicode-3.")),
                &[
                    DirEntry {
                        name: OsString::from("LICENSE_MIT"),
                        path: Default::default(),
                        is_file: true,
                    },
                    DirEntry {
                        name: OsString::from("LICENSE_UNICODE"),
                        path: Default::default(),
                        is_file: true,
                    }
                ]
            )
        );
    }
}
