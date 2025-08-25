use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use crate::license::License;
use crate::{note, warn};
use colored::Colorize;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::collections::HashMap;
use strsim::normalized_levenshtein;

static LICENSE_TEXTS: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| spdx::text::LICENSE_TEXTS.iter().cloned().collect());

#[derive(PartialEq, Debug)]
pub enum LicenseStatus {
    Valid,
    Empty,
    NoneDeclared,
    TooFew,
    TooMany,
    Mismatch(Vec<DirEntry>),
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
            LicenseStatus::Mismatch(license_texts_not_found) => {
                warn!(
                    "found license(s) in {} whose content was not similar to expected - {}",
                    package.normalised_name.bold(),
                    license_texts_not_found
                        .iter()
                        .filter_map(|license| license.name.to_str())
                        .join(",")
                        .dimmed()
                );
            }
        }
    }
}

pub fn validate_licenses(
    file_io: &impl FileIO,
    declared_licenses: &Option<License>,
    actual_licenses: &[DirEntry],
) -> LicenseStatus {
    if actual_licenses.is_empty() {
        return LicenseStatus::Empty;
    }

    let mut license_texts_not_found = actual_licenses.to_vec();

    if let Some(declared_licenses) = declared_licenses {
        for expected_text in declared_licenses
            .requirements()
            .map(|expression| expression.req.license.clone())
            .filter_map(|license| LICENSE_TEXTS.get(license.to_string().as_str()))
        {
            if let Some(entry) = actual_licenses.iter().find(|entry| {
                file_io
                    .read_file(&entry.path)
                    .ok()
                    .is_some_and(|contents| normalized_levenshtein(expected_text, &contents) >= 0.8)
            }) {
                license_texts_not_found.retain(|e| e.path != entry.path);
            }
        }

        if !license_texts_not_found.is_empty() {
            return LicenseStatus::Mismatch(license_texts_not_found);
        }

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
    use crate::file_io::FileIOSpy;
    use std::ffi::OsString;

    #[test]
    fn no_licenses_found() {
        let file_io_spy = FileIOSpy::default();
        assert_eq!(
            LicenseStatus::Empty,
            validate_licenses(&file_io_spy, &Some(License::parse("MIT")), &[])
        );
    }

    #[test]
    fn no_listed_license() {
        let file_io_spy = FileIOSpy::default();
        assert_eq!(
            LicenseStatus::NoneDeclared,
            validate_licenses(
                &file_io_spy,
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
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .push_back(Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                &Some(License::parse("MIT OR Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: Default::default(),
                    is_file: true,
                }]
            )
        );

        file_io_spy
            .read_file
            .returns
            .push_back(Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                &Some(License::parse("MIT/Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: Default::default(),
                    is_file: true,
                }]
            )
        );

        file_io_spy
            .read_file
            .returns
            .push_back(Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string()));
        file_io_spy
            .read_file
            .returns
            .push_back(Ok(LICENSE_TEXTS.get("Unicode-3.0").unwrap().to_string()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
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
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .push_back(Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string()));
        file_io_spy.read_file.returns.push_back(Ok(String::new()));
        assert_eq!(
            LicenseStatus::TooMany,
            validate_licenses(
                &file_io_spy,
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
