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
    #[serde(rename = "additional")]
    Additional(Vec<String>),
    #[serde(rename = "mismatch")]
    Mismatch(Vec<String>),
}

impl LicenseStatus {
    pub fn warn(&self, package: &Package) {
        match self {
            LicenseStatus::Valid => {}
            LicenseStatus::Empty => {
                warn!(
                    "{} - did not find any licenses for {} - {}",
                    "empty".bold(),
                    package.normalised_name.bold(),
                    match &package.url {
                        Some(url) => format!("try looking here: {url}"),
                        None => "no url".to_string(),
                    }
                );
            }
            LicenseStatus::NoneDeclared => {
                note!(
                    "{} - no declared licenses for {}",
                    "none declared".bold(),
                    package.normalised_name.bold()
                );
            }
            LicenseStatus::TooFew => {
                warn!(
                    "{} - did not find as many licenses as declared for {}",
                    "too few".bold(),
                    package.normalised_name.bold()
                );
            }
            LicenseStatus::Additional(extra_licenses_found) => {
                note!(
                    "{} - found all declared licenses for {}, but found additional licenses - {}",
                    "additional".bold(),
                    package.normalised_name.bold(),
                    extra_licenses_found.iter().join(",").bold()
                );
            }
            LicenseStatus::Mismatch(unmatched_licenses) => {
                warn!(
                    "{} - found license(s) in {} whose content was not similar to declared licenses - {}",
                    "mismatch".bold(),
                    package.normalised_name.bold(),
                    unmatched_licenses.iter().join(",").bold()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            LicenseStatus::Additional(vec!["file".to_string()]),
            toml::from_str(r#"additional = ["file"]"#).unwrap()
        );
        assert_eq!(
            LicenseStatus::Mismatch(vec!["file".to_string()]),
            toml::from_str(r#"mismatch = ["file"]"#).unwrap()
        );
    }

    #[test]
    fn invalid_deserialize() {
        assert!(toml::from_str::<LicenseStatus>("invalid").is_err());
        assert!(toml::from_str::<LicenseStatus>("valid").is_err());
    }
}
