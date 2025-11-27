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

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct LicenseStatuses(pub HashMap<Package, LicenseStatus>);

impl LicenseStatuses {
    pub fn any_invalid(&self) -> bool {
        self.0
            .values()
            .any(|status| *status != LicenseStatus::Valid)
    }

    fn to_group_map(
        &self,
    ) -> HashMap<Discriminant<LicenseStatus>, Vec<(&Package, &LicenseStatus)>> {
        self.0
            .iter()
            .map(|(package, status)| (std::mem::discriminant(status), (package, status)))
            .into_group_map()
    }

    fn display_group_item(
        f: &mut Formatter<'_>,
        package: &Package,
        status: &LicenseStatus,
    ) -> std::fmt::Result {
        use LicenseStatus::*;

        write!(f, "   {}", package.normalised_name.bold())?;

        match status {
            Additional(licenses) | Mismatch(licenses) => writeln!(f, " - {}", licenses.join(", ")),
            Empty => writeln!(
                f,
                " - {}",
                match &package.url {
                    None => "no url".to_string(),
                    Some(url) => format!("try looking here: {url}"),
                }
            ),
            _ => writeln!(f),
        }
    }
}

impl Display for LicenseStatuses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (_, items) in self.to_group_map() {
            let Some((_, heading_status)) = items.first() else {
                continue;
            };

            if matches!(heading_status, LicenseStatus::Valid) {
                continue;
            }

            write!(
                f,
                "{}",
                log_message(heading_status.log_level(), &format!("{heading_status}"))
            )?;

            for (package, status) in items {
                Self::display_group_item(f, package, status)?;
            }
        }

        Ok(())
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

    #[test]
    fn display_no_license_statuses() {
        assert!(LicenseStatuses(HashMap::new()).to_string().is_empty());
    }

    #[test]
    fn display_ignores_valid_license_statuses() {
        assert!(
            LicenseStatuses(
                vec![(Package::called("example"), LicenseStatus::Valid)]
                    .into_iter()
                    .collect()
            )
            .to_string()
            .is_empty()
        );
    }

    #[test]
    fn display_package_with_empty_status_and_url() {
        assert_eq!(
            "warning: empty - did not find any licenses for\n   example - try looking here: example.url\n",
            strip_ansi_escapes::strip_str(
                LicenseStatuses(
                    vec![(
                        Package {
                            normalised_name: "example".to_string(),
                            path: Default::default(),
                            url: Some("example.url".to_string()),
                            license: None,
                        },
                        LicenseStatus::Empty
                    )]
                    .into_iter()
                    .collect()
                )
                .to_string()
            )
        );
    }
}
