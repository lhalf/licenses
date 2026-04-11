use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::status::LicenseStatus;
use spdx::detection::scan::Scanner;
use spdx::detection::{Store, TextData};
use std::cmp::Ordering;
use std::sync::LazyLock;

static LICENSE_STORE: LazyLock<Option<Store>> = LazyLock::new(|| Store::load_inline().ok());

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

    let declared_ids = declared_license_ids(declared);
    let unmatched_license_files = unmatched_license_files(file_io, &declared_ids, actual_licenses);

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

fn declared_license_ids(declared: &License) -> Vec<String> {
    declared
        .requirements()
        .map(|req| req.req.license.to_string())
        .collect()
}

fn unmatched_license_files(
    file_io: &impl FileIO,
    declared_ids: &[String],
    actual_licenses: &[DirEntry],
) -> Vec<DirEntry> {
    let Some(store) = LICENSE_STORE.as_ref() else {
        return actual_licenses.to_vec();
    };

    let scanner = Scanner::new(store)
        .confidence_threshold(CONFIDENCE_THRESHOLD)
        .optimize(true)
        .shallow_limit(0.95);

    let mut remaining: Vec<DirEntry> = actual_licenses.to_vec();
    let mut unfound_ids: Vec<&str> = declared_ids.iter().map(String::as_str).collect();

    for entry in actual_licenses {
        let detected_ids = detect_license_ids(file_io, &scanner, entry);

        let matched: Vec<usize> = unfound_ids
            .iter()
            .enumerate()
            .filter_map(|(i, &id)| detected_ids.iter().any(|d| d == id).then_some(i))
            .collect();

        if !matched.is_empty() {
            for i in matched.into_iter().rev() {
                unfound_ids.remove(i);
            }
            remaining.retain(|e| e.name != entry.name);
        }
    }

    remaining
}

fn detect_license_ids(file_io: &impl FileIO, scanner: &Scanner, entry: &DirEntry) -> Vec<String> {
    let Ok(contents) = file_io.read_file(&entry.path) else {
        return vec![];
    };

    let text_data = TextData::from(contents.as_str());
    let result = scanner.scan(&text_data);

    let mut ids = Vec::new();

    if let Some(license) = result.license {
        ids.push(license.name.to_string());
    }

    for contained in &result.containing {
        ids.push(contained.license.name.to_string());
    }

    ids
}

fn found_all_declared_licenses(
    declared: &License,
    actual_licenses: &[DirEntry],
    unmatched_license_files: &[DirEntry],
) -> bool {
    declared.requirements().count() == actual_licenses.len() - unmatched_license_files.len()
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
        file_io_spy
            .read_file
            .returns
            .set([Ok(license_text("MIT")), Ok(license_text("Unicode-3.0"))]);

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
        file_io_spy.read_file.returns.set([
            Ok(license_text("MIT")),
            Ok("not a recognized license".to_string()),
        ]);

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
        file_io_spy.read_file.returns.set([Ok(combined)]);

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
        spdx::text::LICENSE_TEXTS
            .iter()
            .find(|(name, _)| *name == id)
            .unwrap()
            .1
            .to_string()
    }
}
