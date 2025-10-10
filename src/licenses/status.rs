use crate::cargo_metadata::Package;
use crate::{note, warn};
use colored::Colorize;
use itertools::Itertools;
use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize)]
pub enum LicenseStatus {
    #[serde(skip)]
    Valid,
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "none declared")]
    NoneDeclared,
    #[serde(rename = "too few")]
    TooFew,
    #[serde(rename = "too many")]
    TooMany,
    Mismatch(Vec<String>),
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
                    "found license(s) in {} whose content was not similar to declared licenses - {}",
                    package.normalised_name.bold(),
                    license_texts_not_found.iter().join(",").bold()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn valid_deserialize() {
        assert_eq!(
            LicenseStatus::Empty,
            serde_json::from_str(r#""empty""#).unwrap()
        );
        assert_eq!(
            LicenseStatus::NoneDeclared,
            serde_json::from_str(r#""none declared""#).unwrap()
        );
        assert_eq!(
            LicenseStatus::TooFew,
            serde_json::from_str(r#""too few""#).unwrap()
        );
        assert_eq!(
            LicenseStatus::TooMany,
            serde_json::from_str(r#""too many""#).unwrap()
        );
        // TODO
        // assert_eq!(
        //     LicenseStatus::Mismatch(Vec::new()),
        //     serde_json::from_str(r#""mismatch""#).unwrap()
        // );
    }

    #[test]
    fn invalid_deserialize() {
        assert!(serde_json::from_str::<LicenseStatus>(r#""invalid""#).is_err());
        assert!(serde_json::from_str::<LicenseStatus>(r#""valid""#).is_err());
    }
}
