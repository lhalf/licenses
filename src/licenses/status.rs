use crate::cargo_metadata::Package;
use crate::{note, warn};
use colored::Colorize;
use itertools::Itertools;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

#[derive(PartialEq, Debug)]
pub enum LicenseStatus {
    Valid,
    Empty,
    NoneDeclared,
    TooFew,
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

impl<'de> Deserialize<'de> for LicenseStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LicenseStatusVisitor;

        impl<'de> Visitor<'de> for LicenseStatusVisitor {
            type Value = LicenseStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"a license status string: "empty", "none declared", "too few", "too many" or "mismatch""#)
            }

            fn visit_str<E>(self, value: &str) -> Result<LicenseStatus, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "empty" => Ok(LicenseStatus::Empty),
                    "none declared" => Ok(LicenseStatus::NoneDeclared),
                    "too few" => Ok(LicenseStatus::TooFew),
                    "too many" => Ok(LicenseStatus::TooMany),
                    "mismatch" => Ok(LicenseStatus::Mismatch(Vec::new())),
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(value),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_str(LicenseStatusVisitor)
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
        assert_eq!(
            LicenseStatus::Mismatch(Vec::new()),
            serde_json::from_str(r#""mismatch""#).unwrap()
        );
    }

    #[test]
    fn invalid_deserialize() {
        assert!(serde_json::from_str::<LicenseStatus>(r#""invalid""#).is_err());
        assert!(serde_json::from_str::<LicenseStatus>(r#""valid""#).is_err());
    }
}
