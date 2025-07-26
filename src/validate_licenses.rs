use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::{note, warn};
use colored::Colorize;

#[derive(PartialEq, Debug)]
pub enum LicenseStatus {
    Valid,
    Empty,
    NoListing,
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
            LicenseStatus::NoListing => {
                note!("no listed licenses for {}", package.normalised_name.bold());
            }
        }
    }
}

pub fn validate_licenses(package: &Package, actual_licenses: &[DirEntry]) -> LicenseStatus {
    if actual_licenses.is_empty() {
        return LicenseStatus::Empty;
    }

    if package.license.is_none() {
        return LicenseStatus::NoListing;
    }

    LicenseStatus::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_licenses_found() {
        assert_eq!(
            LicenseStatus::Empty,
            validate_licenses(&Package::default(), &[])
        );
    }

    #[test]
    fn no_listed_license() {
        assert_eq!(
            LicenseStatus::NoListing,
            validate_licenses(
                &Package::default(),
                &[DirEntry {
                    name: Default::default(),
                    path: Default::default(),
                    is_file: false,
                }]
            )
        );
    }
}
