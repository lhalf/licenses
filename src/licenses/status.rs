use crate::cargo_metadata::Package;
use crate::log::LogLevel;
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
    pub fn log_message(&self, package: &Package) -> String {
        match self {
            LicenseStatus::Valid => String::new(),
            LicenseStatus::Empty => {
                format!(
                    "{} - did not find any licenses for {} - {}",
                    "empty".bold(),
                    package.normalised_name.bold(),
                    match &package.url {
                        Some(url) => format!("try looking here: {url}"),
                        None => "no url".to_string(),
                    }
                )
            }
            LicenseStatus::NoneDeclared => {
                format!(
                    "{} - no declared licenses for {}",
                    "none declared".bold(),
                    package.normalised_name.bold()
                )
            }
            LicenseStatus::TooFew => {
                format!(
                    "{} - did not find as many licenses as declared for {}",
                    "too few".bold(),
                    package.normalised_name.bold()
                )
            }
            LicenseStatus::Additional(extra_licenses_found) => {
                format!(
                    "{} - found all declared licenses for {}, but found additional licenses - {}",
                    "additional".bold(),
                    package.normalised_name.bold(),
                    extra_licenses_found.iter().join(",").bold()
                )
            }
            LicenseStatus::Mismatch(unmatched_licenses) => {
                format!(
                    "{} - found license(s) in {} whose content was not similar to declared licenses - {}",
                    "mismatch".bold(),
                    package.normalised_name.bold(),
                    unmatched_licenses.iter().join(",").bold()
                )
            }
        }
    }

    pub fn log_level(&self) -> LogLevel {
        match self {
            LicenseStatus::Additional(_) | LicenseStatus::NoneDeclared => LogLevel::Note,
            _ => LogLevel::Warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strip_ansi_escapes::strip_str;

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

    #[test]
    fn log_empty_license() {
        assert_eq!(
            strip_str(LicenseStatus::Empty.log_message(&Package::called("example"))),
            "empty - did not find any licenses for example - no url"
        );
    }

    #[test]
    fn log_none_declared() {
        assert_eq!(
            strip_str(LicenseStatus::NoneDeclared.log_message(&Package::called("example"))),
            "none declared - no declared licenses for example"
        );
    }

    #[test]
    fn log_too_few() {
        assert_eq!(
            strip_str(LicenseStatus::TooFew.log_message(&Package::called("example"))),
            "too few - did not find as many licenses as declared for example"
        );
    }

    #[test]
    fn log_additional() {
        assert_eq!(
            strip_str(
                LicenseStatus::Additional(vec!["COPYRIGHT".to_string()])
                    .log_message(&Package::called("example"))
            ),
            "additional - found all declared licenses for example, but found additional licenses - COPYRIGHT"
        );
    }

    #[test]
    fn log_mismatch() {
        assert_eq!(
            strip_str(
                LicenseStatus::Mismatch(vec!["LICENSE".to_string()])
                    .log_message(&Package::called("example"))
            ),
            "mismatch - found license(s) in example whose content was not similar to declared licenses - LICENSE"
        );
    }
}
