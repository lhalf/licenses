use crate::cargo_metadata::Package;
use crate::log::{LogLevel, log_message};
use colored::Colorize;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem::Discriminant;

#[derive(PartialEq, Eq, Hash, Debug, Deserialize)]
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

impl Display for LicenseStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseStatus::Valid => Ok(()),
            LicenseStatus::Empty => {
                writeln!(f, "{} - did not find any licenses for", "empty".bold())
            }
            LicenseStatus::NoneDeclared => {
                writeln!(f, "{} - no declared licenses for", "none declared".bold())
            }
            LicenseStatus::TooFew => writeln!(
                f,
                "{} - did not find as many licenses as declared for",
                "too few".bold()
            ),
            LicenseStatus::Additional(_) => writeln!(
                f,
                "{} - found all declared licenses, but found additional licenses for",
                "additional".bold()
            ),
            LicenseStatus::Mismatch(_) => writeln!(
                f,
                "{} - found license(s) whose content was not similar to declared licenses for",
                "mismatch".bold()
            ),
        }
    }
}

impl LicenseStatus {
    pub fn log_level(&self) -> LogLevel {
        match self {
            LicenseStatus::Additional(_) | LicenseStatus::NoneDeclared => LogLevel::Note,
            _ => LogLevel::Warning,
        }
    }
}

pub struct LicenseStatuses(pub HashMap<Package, LicenseStatus>);

impl LicenseStatuses {
    pub fn any_invalid(&self) -> bool {
        self.0
            .values()
            .any(|status| *status != LicenseStatus::Valid)
    }

    pub fn group_map(
        &self,
    ) -> HashMap<Discriminant<LicenseStatus>, Vec<(&Package, &LicenseStatus)>> {
        self.0
            .iter()
            .map(|(package, status)| (std::mem::discriminant(status), (package, status)))
            .into_group_map()
    }
}

impl Display for LicenseStatuses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.group_map().into_iter().try_for_each(|(_, items)| {
            let Some((_, heading_status)) = items.first() else {
                return Ok(());
            };

            if matches!(heading_status, LicenseStatus::Valid) {
                return Ok(());
            }

            write!(
                f,
                "{}",
                log_message(heading_status.log_level(), &format!("{heading_status}"))
            )?;

            for (package, status) in items {
                match status {
                    LicenseStatus::Additional(licenses) | LicenseStatus::Mismatch(licenses) => {
                        writeln!(
                            f,
                            "   {} - {}",
                            package.normalised_name.bold(),
                            licenses.join(",")
                        )
                    }
                    _ => writeln!(f, "   {}", package.normalised_name.bold()),
                }?;
            }

            Ok(())
        })
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
