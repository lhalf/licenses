use crate::cargo_metadata::Package;
use crate::config::CrateConfig;
use crate::file_io::{DirEntry, FileIO};
use crate::licenses::License;
use crate::licenses::status::LicenseStatus;
use crate::licenses::status::LicenseStatuses;
use crate::licenses::validate::validate_licenses;
use std::collections::HashMap;

pub fn check_licenses(
    file_io: &impl FileIO,
    all_licenses: HashMap<Package, Vec<DirEntry>>,
    crate_configs: &HashMap<String, CrateConfig>,
) -> LicenseStatuses {
    LicenseStatuses(
        all_licenses
            .into_iter()
            .map(|(package, licenses)| {
                (
                    package.clone(),
                    license_status_after_allowed(
                        validate_licenses(
                            file_io,
                            &package.license.as_deref().map(License::parse),
                            &licenses,
                        ),
                        &package,
                        crate_configs,
                    ),
                )
            })
            .collect(),
    )
}

fn license_status_after_allowed(
    license_status: LicenseStatus,
    package: &Package,
    crate_configs: &HashMap<String, CrateConfig>,
) -> LicenseStatus {
    match crate_configs.get(&package.normalised_name) {
        Some(config) => match &config.allow {
            Some(allowed_status) if *allowed_status == license_status => LicenseStatus::Valid,
            _ => license_status,
        },
        None => license_status,
    }
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::config::CrateConfig;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::licenses::check::check_licenses;
    use crate::licenses::status::{LicenseStatus, LicenseStatuses};
    use crate::licenses::validate::LICENSE_TEXTS;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

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

        let expected_statuses = LicenseStatuses(
            vec![
                (Package::called("bad"), LicenseStatus::Empty),
                (
                    Package {
                        normalised_name: "good".to_string(),
                        path: Default::default(),
                        url: None,
                        license: Some("MIT".to_string()),
                    },
                    LicenseStatus::Valid,
                ),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(
            expected_statuses,
            check_licenses(&file_io_spy, all_licenses, &HashMap::new())
        );
    }

    #[test]
    fn license_status_that_has_been_allowed_has_license_status_valid() {
        let file_io_spy = FileIOSpy::default();

        let all_licenses: HashMap<_, _> = [(
            Package {
                normalised_name: "some_crate".to_string(),
                path: Default::default(),
                url: None,
                license: Some("MIT".to_string()),
            },
            vec![],
        )]
        .into_iter()
        .collect();

        // errors with no allowed status
        assert!(check_licenses(&file_io_spy, all_licenses.clone(), &HashMap::new()).any_invalid());

        let config = [(
            "some_crate".to_string(),
            CrateConfig {
                skip: vec![],
                allow: Some(LicenseStatus::TooFew),
                include: vec![],
            },
        )]
        .into_iter()
        .collect();

        // errors when allowed status is incorrect
        assert!(check_licenses(&file_io_spy, all_licenses.clone(), &config).any_invalid());

        let config = [(
            "some_crate".to_string(),
            CrateConfig {
                skip: vec![],
                allow: Some(LicenseStatus::Empty),
                include: vec![],
            },
        )]
        .into_iter()
        .collect();

        // fine when status is allowed
        assert!(!check_licenses(&file_io_spy, all_licenses, &config).any_invalid());
    }
}
