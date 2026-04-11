use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::status::LicenseStatus;
use spdx::detection::TextData;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static LICENSE_TEXTS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| spdx::text::LICENSE_TEXTS.iter().copied().collect());

const CONFIDENCE_THRESHOLD: f32 = 0.8;

pub fn validate_licenses(
    file_io: &impl FileIO,
    declared_licenses: Option<&License>,
    actual_licenses: &[DirEntry],
) -> LicenseStatus {
    if actual_licenses.is_empty() {
        return LicenseStatus::Empty;
    }

    let Some(declared) = declared_licenses else {
        return LicenseStatus::NoneDeclared;
    };

    let expected_texts = expected_texts_from_declared(declared);
    let unmatched_license_files =
        unmatched_license_files(file_io, &expected_texts, actual_licenses);

    if !unmatched_license_files.is_empty()
        && !found_all_declared_licenses(declared, actual_licenses, &unmatched_license_files)
    {
        return LicenseStatus::Mismatch(to_file_names(unmatched_license_files));
    }

    match actual_licenses.len().cmp(&declared.requirements().count()) {
        Ordering::Equal => LicenseStatus::Valid,
        Ordering::Less => LicenseStatus::TooFew,
        Ordering::Greater => LicenseStatus::Additional(to_file_names(unmatched_license_files)),
    }
}

fn expected_texts_from_declared(declared: &License) -> Vec<TextData> {
    declared
        .requirements()
        .filter_map(|expression| {
            LICENSE_TEXTS
                .get(expression.req.license.to_string().as_str())
                .map(|&text| TextData::new(text))
        })
        .collect()
}

fn unmatched_license_files(
    file_io: &impl FileIO,
    expected_texts: &[TextData],
    actual_licenses: &[DirEntry],
) -> Vec<DirEntry> {
    let mut remaining: Vec<DirEntry> = actual_licenses.to_vec();

    for expected in expected_texts {
        if let Some(entry) = find_matching_entry(file_io, expected, &remaining) {
            remaining.retain(|e| e.name != entry.name);
        }
    }

    remaining
}

fn found_all_declared_licenses(
    declared: &License,
    actual_licenses: &[DirEntry],
    unmatched_license_files: &[DirEntry],
) -> bool {
    declared.requirements().count() == actual_licenses.len() - unmatched_license_files.len()
}

fn find_matching_entry(
    file_io: &impl FileIO,
    expected: &TextData,
    remaining_licenses: &[DirEntry],
) -> Option<DirEntry> {
    remaining_licenses
        .iter()
        .find(|entry| {
            file_io.read_file(&entry.path).ok().is_some_and(|contents| {
                TextData::from(contents.as_str()).match_score(expected) >= CONFIDENCE_THRESHOLD
            })
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
    use std::path::PathBuf;

    #[test]
    fn failure_to_read_license_file_causes_mismatch() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        assert_eq!(
            LicenseStatus::Mismatch(vec!["LICENSE".to_string()]),
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT")),
                &[DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn no_licenses_found() {
        let file_io_spy = FileIOSpy::default();
        assert_eq!(
            LicenseStatus::Empty,
            validate_licenses(&file_io_spy, Some(&License::parse("MIT")), &[])
        );
    }

    #[test]
    fn no_declared_license() {
        let file_io_spy = FileIOSpy::default();
        assert_eq!(
            LicenseStatus::NoneDeclared,
            validate_licenses(
                &file_io_spy,
                None,
                &[DirEntry {
                    name: OsString::new(),
                    path: PathBuf::new(),
                    is_file: false,
                }]
            )
        );
    }

    #[test]
    fn too_few_licenses() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([Ok(license_text("MIT"))]);

        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT OR Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn too_few_licenses_non_standard_seperator() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([Ok(license_text("MIT"))]);

        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT/Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn too_few_licenses_complex_requirements() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([
            Ok(license_text("MIT")),
            Ok(license_text("Unicode-3.0")),
            Ok(license_text("Unicode-3.0")),
        ]);

        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("(MIT OR Apache-2.0) AND Unicode-3.0")),
                &[
                    DirEntry {
                        name: OsString::from("LICENSE_MIT"),
                        path: PathBuf::new(),
                        is_file: true,
                    },
                    DirEntry {
                        name: OsString::from("LICENSE_UNICODE"),
                        path: PathBuf::new(),
                        is_file: true,
                    }
                ]
            )
        );
    }

    #[test]
    fn additional_licenses() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([Ok(license_text("MIT"))]);

        assert_eq!(
            LicenseStatus::Additional(vec!["COPYING".to_string()]),
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT")),
                &[
                    DirEntry {
                        name: OsString::from("LICENSE_MIT"),
                        path: PathBuf::new(),
                        is_file: true,
                    },
                    DirEntry {
                        name: OsString::from("COPYING"),
                        path: PathBuf::new(),
                        is_file: true,
                    }
                ]
            )
        );
    }

    #[test]
    fn license_content_mismatch() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Ok(license_text("Apache-2.0"))]);

        assert_eq!(
            LicenseStatus::Mismatch(vec!["LICENSE_MIT".to_string()]),
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT")),
                &[DirEntry {
                    name: OsString::from("LICENSE_MIT"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn valid_license() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_file.returns.set([Ok(license_text("MIT"))]);

        assert_eq!(
            LicenseStatus::Valid,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT")),
                &[DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn valid_license_with_different_copyright_header() {
        let file_io_spy = FileIOSpy::default();
        let mit_with_custom_copyright = license_text("MIT").replacen("[year]", "2026", 1).replacen(
            "[fullname]",
            "Custom Author Name",
            1,
        );
        file_io_spy
            .read_file
            .returns
            .set([Ok(mit_with_custom_copyright)]);

        assert_eq!(
            LicenseStatus::Valid,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT")),
                &[DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    #[test]
    fn single_file_containing_both_declared_licenses() {
        let file_io_spy = FileIOSpy::default();
        let combined = format!(
            "{}\n\n---\n\n{}",
            license_text("MIT"),
            license_text("Apache-2.0")
        );
        file_io_spy
            .read_file
            .returns
            .set([Ok(combined.clone()), Ok(combined)]);

        assert_eq!(
            LicenseStatus::TooFew,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT OR Apache-2.0")),
                &[DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }

    fn license_text(id: &str) -> String {
        LICENSE_TEXTS.get(id).unwrap().to_string()
    }

    #[test]
    fn valid_dual_license() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Ok(license_text("MIT")), Ok(license_text("Apache-2.0"))]);

        assert_eq!(
            LicenseStatus::Valid,
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("MIT OR Apache-2.0")),
                &[
                    DirEntry {
                        name: OsString::from("LICENSE-MIT"),
                        path: PathBuf::new(),
                        is_file: true,
                    },
                    DirEntry {
                        name: OsString::from("LICENSE-APACHE"),
                        path: PathBuf::new(),
                        is_file: true,
                    }
                ]
            )
        );
    }

    #[test]
    fn unknown_declared_license_with_files_present() {
        let file_io_spy = FileIOSpy::default();

        assert_eq!(
            LicenseStatus::Additional(vec!["LICENSE".to_string()]),
            validate_licenses(
                &file_io_spy,
                Some(&License::parse("not-a-real-license")),
                &[DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::new(),
                    is_file: true,
                }]
            )
        );
    }
}
