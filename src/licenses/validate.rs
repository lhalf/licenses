use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::status::LicenseStatus;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::collections::HashMap;
use strsim::normalized_levenshtein;

pub static LICENSE_TEXTS: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| spdx::text::LICENSE_TEXTS.iter().cloned().collect());

pub fn validate_licenses(
    file_io: &impl FileIO,
    declared_licenses: &Option<License>,
    actual_licenses: &[DirEntry],
) -> LicenseStatus {
    if actual_licenses.is_empty() {
        return LicenseStatus::Empty;
    }

    let Some(declared) = declared_licenses else {
        return LicenseStatus::NoneDeclared;
    };

    let expected_texts = expected_texts_from_declared(declared);
    let unmatched = unmatched_licenses(file_io, &expected_texts, actual_licenses);

    if !unmatched.is_empty() {
        return LicenseStatus::Mismatch(to_file_names(unmatched));
    }

    match actual_licenses.len().cmp(&declared.requirements().count()) {
        Ordering::Equal => LicenseStatus::Valid,
        Ordering::Less => LicenseStatus::TooFew,
        Ordering::Greater => LicenseStatus::TooMany,
    }
}

fn expected_texts_from_declared(declared: &License) -> Vec<&'static str> {
    declared
        .requirements()
        .filter_map(|expression| {
            LICENSE_TEXTS
                .get(expression.req.license.to_string().as_str())
                .copied()
        })
        .collect()
}

fn unmatched_licenses(
    file_io: &impl FileIO,
    expected_texts: &[&str],
    actual_licenses: &[DirEntry],
) -> Vec<DirEntry> {
    let mut remaining: Vec<DirEntry> = actual_licenses.to_vec();

    for &expected in expected_texts {
        if let Some(entry) = find_matching_entry(file_io, expected, &remaining) {
            remaining.retain(|e| e.name != entry.name);
        }
    }

    remaining
}

fn find_matching_entry(
    file_io: &impl FileIO,
    expected_text: &str,
    remaining_licenses: &[DirEntry],
) -> Option<DirEntry> {
    remaining_licenses
        .iter()
        .find(|entry| {
            file_io
                .read_file(&entry.path)
                .ok()
                .is_some_and(|contents| normalized_levenshtein(expected_text, &contents) >= 0.8)
        })
        .cloned()
}

fn to_file_names(entries: Vec<DirEntry>) -> Vec<String> {
    entries
        .into_iter()
        .map(|entry| entry.name.to_string_lossy().to_string())
        .collect()
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
    fn no_declared_license() {
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
            .set([Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string())]);

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
    }

    #[test]
    fn too_few_licenses_non_standard_seperator() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string())]);

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
    }

    #[test]
    fn too_few_licenses_complex_requirements() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([
            Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string()),
            Ok(LICENSE_TEXTS.get("Unicode-3.0").unwrap().to_string()),
            Ok(LICENSE_TEXTS.get("Unicode-3.0").unwrap().to_string()),
        ]);

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

    // #[test]
    // fn too_many_licenses() {
    //     let file_io_spy = FileIOSpy::default();
    //     file_io_spy
    //         .read_file
    //         .returns
    //         .set([Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string())]);
    //
    //     assert_eq!(
    //         LicenseStatus::TooMany,
    //         validate_licenses(
    //             &file_io_spy,
    //             &Some(License::parse("MIT")),
    //             &[
    //                 DirEntry {
    //                     name: OsString::from("LICENSE_MIT"),
    //                     path: Default::default(),
    //                     is_file: true,
    //                 },
    //                 DirEntry {
    //                     name: OsString::from("COPYING"),
    //                     path: Default::default(),
    //                     is_file: true,
    //                 }
    //             ]
    //         )
    //     );
    // }

    #[test]
    fn license_content_mismatch() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Ok(LICENSE_TEXTS.get("Apache-2.0").unwrap().to_string())]);

        assert_eq!(
            LicenseStatus::Mismatch(vec!["LICENSE_MIT".to_string()]),
            validate_licenses(
                &file_io_spy,
                &Some(License::parse("MIT")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: Default::default(),
                    is_file: true,
                }]
            )
        );
    }
}
