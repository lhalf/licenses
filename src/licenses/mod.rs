pub mod check;
pub mod collect;
pub mod copy;
pub mod diff;
pub mod is_license;
pub mod status;
pub mod subcommand;
pub mod summarise;
pub mod unused;
pub mod validate;

use itertools::Itertools;
use serde::{Serialize, Serializer};
use spdx::expression::ExpressionReq;
use spdx::{Expression, LicenseReq, ParseMode};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum License {
    Known(Expression),
    Unknown(String),
}

impl License {
    pub fn parse(license: &str) -> Self {
        fn parse(expression: &str) -> License {
            Expression::parse_mode(expression, ParseMode::LAX)
                .map_or_else(|_| License::Unknown(expression.to_string()), License::Known)
        }

        match Expression::canonicalize(license) {
            Ok(Some(expression)) => parse(&expression),
            Ok(None) => parse(license),
            Err(_) => Self::Unknown(license.to_string()),
        }
    }

    pub fn requirements(&self) -> Box<dyn Iterator<Item = &ExpressionReq> + '_> {
        match self {
            Self::Known(expression) => Box::new(expression.requirements()),
            Self::Unknown(_) => Box::new(Vec::<&ExpressionReq>::new().into_iter()),
        }
    }
}

impl PartialEq for License {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Known(expression_1), Self::Known(expression_2)) => {
                sorted_requirements(expression_1) == sorted_requirements(expression_2)
            }
            (Self::Unknown(license_1), Self::Unknown(license_2)) => license_1 == license_2,
            _ => false,
        }
    }
}

impl Eq for License {}

impl Hash for License {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Known(expression) => {
                for req in sorted_requirements(expression) {
                    req.to_string().hash(state);
                }
            }
            Self::Unknown(license) => license.hash(state),
        }
    }
}

impl Serialize for License {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Known(expression) => serializer.serialize_str(expression.as_ref()),
            Self::Unknown(license) => serializer.serialize_str(license),
        }
    }
}

impl Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Known(expression) => expression.to_string(),
                Self::Unknown(license) => license.clone(),
            }
        )
    }
}

fn sorted_requirements(expression: &Expression) -> Vec<LicenseReq> {
    expression
        .requirements()
        .map(|req| req.req.clone())
        .sorted()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn licenses_with_slash_are_equal() {
        assert_eq!(
            License::parse("MIT OR Apache-2.0"),
            License::parse("Apache-2.0/MIT")
        );
    }

    #[test]
    fn licenses_in_a_different_order_are_equal() {
        assert_eq!(
            License::parse("MIT OR Apache-2.0"),
            License::parse("Apache-2.0 OR MIT")
        );
    }

    #[test]
    fn licenses_with_parentheses_are_equal() {
        assert_eq!(
            License::parse("MIT OR Apache-2.0"),
            License::parse("(MIT OR Apache-2.0)")
        );
    }

    #[test]
    fn licenses_with_with_clause_are_not_equal() {
        assert_ne!(
            License::parse("(MIT WITH Bison-exception-2.2) OR Apache-2.0"),
            License::parse("MIT OR (Apache-2.0 WITH Bison-exception-2.2)")
        );
    }

    #[test]
    fn unknown_license_is_parsed_when_invalid_spdx() {
        assert!(matches!(
            License::parse("not-a-real-license"),
            License::Unknown(_)
        ));
    }

    #[test]
    fn known_license_is_parsed_for_valid_spdx() {
        assert!(matches!(License::parse("MIT"), License::Known(_)));
    }

    #[test]
    fn unknown_licenses_with_same_text_are_equal() {
        assert_eq!(
            License::parse("custom-license-1.0"),
            License::parse("custom-license-1.0")
        );
    }

    #[test]
    fn unknown_licenses_with_different_text_are_not_equal() {
        assert_ne!(
            License::parse("custom-license-1.0"),
            License::parse("custom-license-2.0")
        );
    }

    #[test]
    fn known_and_unknown_licenses_are_not_equal() {
        assert_ne!(License::parse("MIT"), License::parse("not-a-real-license"));
    }

    #[test]
    fn known_license_has_requirements() {
        assert!(License::parse("MIT").requirements().count() > 0);
    }

    #[test]
    fn unknown_license_has_no_requirements() {
        assert_eq!(
            0,
            License::parse("not-a-real-license").requirements().count()
        );
    }

    #[test]
    fn dual_license_has_two_requirements() {
        assert_eq!(
            2,
            License::parse("MIT OR Apache-2.0").requirements().count()
        );
    }

    #[test]
    fn display_known_license() {
        assert_eq!("MIT", License::parse("MIT").to_string());
    }

    #[test]
    fn display_unknown_license() {
        assert_eq!(
            "not-a-real-license",
            License::parse("not-a-real-license").to_string()
        );
    }

    #[test]
    fn serialize_known_license() {
        assert_eq!(
            r#""MIT""#,
            serde_json::to_string(&License::parse("MIT")).unwrap()
        );
    }

    #[test]
    fn serialize_unknown_license() {
        assert_eq!(
            r#""not-a-real-license""#,
            serde_json::to_string(&License::parse("not-a-real-license")).unwrap()
        );
    }

    #[test]
    fn equal_licenses_have_equal_hashes() {
        use std::collections::hash_map::DefaultHasher;

        let hash = |license: License| {
            let mut hasher = DefaultHasher::new();
            license.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(
            hash(License::parse("MIT OR Apache-2.0")),
            hash(License::parse("Apache-2.0 OR MIT"))
        );
    }

    #[test]
    fn licenses_with_parentheses_have_equal_hashes() {
        use std::collections::hash_map::DefaultHasher;

        let hash = |license: License| {
            let mut hasher = DefaultHasher::new();
            license.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(
            hash(License::parse("MIT OR Apache-2.0")),
            hash(License::parse("(MIT OR Apache-2.0)"))
        );
    }

    #[test]
    fn licenses_with_different_with_clauses_have_different_hashes() {
        use std::collections::hash_map::DefaultHasher;

        let hash = |license: License| {
            let mut hasher = DefaultHasher::new();
            license.hash(&mut hasher);
            hasher.finish()
        };

        assert_ne!(
            hash(License::parse(
                "(MIT WITH Bison-exception-2.2) OR Apache-2.0"
            )),
            hash(License::parse(
                "MIT OR (Apache-2.0 WITH Bison-exception-2.2)"
            ))
        );
    }

    #[test]
    fn complex_nested_parentheses_are_equal() {
        assert_eq!(
            License::parse("MIT OR Apache-2.0 OR BSD-2-Clause"),
            License::parse("(MIT OR Apache-2.0) OR BSD-2-Clause")
        );
    }

    #[test]
    fn three_licenses_in_different_order_are_equal() {
        assert_eq!(
            License::parse("MIT OR Apache-2.0 OR BSD-2-Clause"),
            License::parse("BSD-2-Clause OR MIT OR Apache-2.0")
        );
    }

    #[test]
    fn same_with_clause_in_different_order_are_equal() {
        assert_eq!(
            License::parse("(MIT WITH Bison-exception-2.2) OR Apache-2.0"),
            License::parse("Apache-2.0 OR (MIT WITH Bison-exception-2.2)")
        );
    }
}
