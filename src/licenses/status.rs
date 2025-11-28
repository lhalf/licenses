use crate::cargo_metadata::Package;
use crate::log::warning;
use colored::Colorize;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::mem::discriminant;

#[derive(PartialEq, Eq, Hash, Debug, Deserialize, PartialOrd, Ord)]
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
                writeln!(f, "{} - did not find any licenses for:", "empty".bold())
            }
            LicenseStatus::NoneDeclared => {
                writeln!(f, "{} - no declared licenses for:", "none declared".bold())
            }
            LicenseStatus::TooFew => writeln!(
                f,
                "{} - did not find as many licenses as declared for:",
                "too few".bold()
            ),
            LicenseStatus::Additional(_) => writeln!(
                f,
                "{} - found all declared licenses, but found additional licenses for:",
                "additional".bold()
            ),
            LicenseStatus::Mismatch(_) => writeln!(
                f,
                "{} - found license(s) whose content was not similar to declared licenses for:",
                "mismatch".bold()
            ),
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

    fn invalid_statuses(&self) -> impl Iterator<Item = &LicenseStatus> {
        let mut seen = HashSet::new();
        self.0
            .values()
            .filter(|status| !matches!(status, LicenseStatus::Valid))
            .filter(move |status| seen.insert(discriminant(*status)))
    }

    fn packages_with_status(
        &self,
        license_status: &LicenseStatus,
    ) -> impl Iterator<Item = (&Package, &LicenseStatus)> {
        self.0
            .iter()
            .filter(move |(_, status)| discriminant(*status) == discriminant(license_status))
            .sorted()
    }

    fn display_status_section(
        &self,
        f: &mut Formatter<'_>,
        license_status: &LicenseStatus,
    ) -> std::fmt::Result {
        write!(f, "{}", warning(&format!("{license_status}")))?;
        for (package, status) in self.packages_with_status(license_status) {
            Self::display_status_item(f, package, status)?;
        }
        Ok(())
    }

    fn display_status_item(
        f: &mut Formatter<'_>,
        package: &Package,
        status: &LicenseStatus,
    ) -> std::fmt::Result {
        use LicenseStatus::*;

        write!(f, "\t{}", package.normalised_name.bold())?;

        match status {
            Additional(licenses) | Mismatch(licenses) => {
                writeln!(f, " - {}", licenses.iter().sorted().join(", "))
            }
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
        for status in self.invalid_statuses() {
            self.display_status_section(f, status)?;
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
    fn display_package_with_empty_status_and_with_without_url() {
        assert_eq!(
            "warning: empty - did not find any licenses for:\nexample - try looking here: example.url\nexample2 - no url\n",
            strip_ansi_escapes::strip_str(
                LicenseStatuses(
                    vec![
                        (
                            Package {
                                normalised_name: "example".to_string(),
                                path: Default::default(),
                                url: Some("example.url".to_string()),
                                license: None,
                            },
                            LicenseStatus::Empty
                        ),
                        (
                            Package {
                                normalised_name: "example2".to_string(),
                                path: Default::default(),
                                url: None,
                                license: None,
                            },
                            LicenseStatus::Empty
                        )
                    ]
                    .into_iter()
                    .collect()
                )
                .to_string()
            )
        );
    }

    #[test]
    fn display_groups_multiple_packages_under_the_same_status_and_in_order() {
        assert_eq!(
            "warning: none declared - no declared licenses for:\na\nb\nc\n",
            strip_ansi_escapes::strip_str(
                LicenseStatuses(
                    vec![
                        (Package::called("b"), LicenseStatus::NoneDeclared),
                        (Package::called("a"), LicenseStatus::NoneDeclared),
                        (Package::called("c"), LicenseStatus::NoneDeclared)
                    ]
                    .into_iter()
                    .collect()
                )
                .to_string()
            )
        );
    }

    #[test]
    fn display_additional_licenses_list_in_order() {
        assert_eq!(
            "warning: additional - found all declared licenses, but found additional licenses for:\nexample - a, b, c\n",
            strip_ansi_escapes::strip_str(
                LicenseStatuses(
                    vec![(
                        Package::called("example"),
                        LicenseStatus::Additional(vec![
                            "a".to_string(),
                            "b".to_string(),
                            "c".to_string()
                        ])
                    ),]
                    .into_iter()
                    .collect()
                )
                .to_string()
            )
        );
    }
}
