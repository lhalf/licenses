use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::license::License;
use crate::{note, warn};
use colored::Colorize;
use std::cmp::Ordering;

#[derive(PartialEq, Debug)]
pub enum LicenseStatus {
    Valid,
    Empty,
    NoneDeclared,
    TooFew,
    TooMany,
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
            LicenseStatus::TooFew => {
                warn!(
                    "did not find as many licenses as declared for {}",
                    package.normalised_name.bold()
                );
            }
            LicenseStatus::TooMany => {
                note!(
                    "found more licenses than declared for {}",
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

    if let Some(declared_licenses) = declared_licenses {
        match actual_licenses
            .len()
            .cmp(&declared_licenses.requirements().count())
        {
            Ordering::Equal => LicenseStatus::Valid,
            Ordering::Less => LicenseStatus::TooFew,
            Ordering::Greater => LicenseStatus::TooMany,
        }
    } else {
        LicenseStatus::NoneDeclared
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
    fn too_few_licenses() {
        assert_eq!(
            LicenseStatus::TooFew,
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
            LicenseStatus::TooFew,
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
            LicenseStatus::TooFew,
            validate_licenses(
                &Some(License::parse("(MIT OR Apache-2.0) AND Unicode-3.0")),
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

    #[test]
    fn too_many_licenses() {
        assert_eq!(
            LicenseStatus::TooMany,
            validate_licenses(
                &Some(License::parse("MIT")),
                &[
                    DirEntry {
                        name: OsString::from("LICENSE_MIT"),
                        path: Default::default(),
                        is_file: true,
                    },
                    DirEntry {
                        name: OsString::from("COPYING"),
                        path: Default::default(),
                        is_file: true,
                    }
                ]
            )
        );
    }
}
