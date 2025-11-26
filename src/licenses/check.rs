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
