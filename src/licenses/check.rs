use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::status::LicenseStatus;
use crate::licenses::validate::validate_licenses;
use std::collections::HashMap;

pub fn check_licenses(
    file_io: &impl FileIO,
    all_licenses: HashMap<Package, Vec<DirEntry>>,
) -> anyhow::Result<()> {
    let mut issues_found = false;

    for (package, licenses) in all_licenses {
        match validate_licenses(
            file_io,
            &package.license.as_deref().map(License::parse),
            &licenses,
        ) {
            LicenseStatus::Valid => continue,
            status => {
                status.warn(&package);
                issues_found = true;
            }
        }
    }

    if issues_found {
        Err(anyhow::anyhow!("licenses had inconsistencies"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::licenses::check::check_licenses;
    use crate::licenses::validate::LICENSE_TEXTS;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn failure_to_read_file_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        let all_licenses = [(
            Package {
                normalised_name: "example".to_string(),
                path: Default::default(),
                url: None,
                license: Some("MIT".to_string()),
            },
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: false,
            }],
        )]
        .into_iter()
        .collect();

        assert!(check_licenses(&file_io_spy, all_licenses).is_err());
    }

    #[test]
    fn checks_all_packages_even_if_the_first_has_issues() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Ok(LICENSE_TEXTS.get("MIT").unwrap().to_string())]);

        let all_licenses = [
            (
                Package {
                    normalised_name: "bad".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                vec![],
            ),
            (
                Package {
                    normalised_name: "good".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("MIT".to_string()),
                },
                vec![DirEntry {
                    name: OsString::from("LICENSE"),
                    path: PathBuf::from("example/LICENSE"),
                    is_file: true,
                }],
            ),
        ]
        .into_iter()
        .collect();

        assert!(check_licenses(&file_io_spy, all_licenses).is_err());
    }
}
