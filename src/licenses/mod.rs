pub mod check;
pub mod collect;
pub mod copy;
pub mod diff;
pub mod is_license;
pub mod status;
pub mod subcommand;
pub mod summarise;
pub mod validate;

use itertools::Itertools;
use serde::{Serialize, Serializer};
use spdx::expression::ExpressionReq;
use spdx::{Expression, ParseMode};
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
                sorted_expression(expression_1) == sorted_expression(expression_2)
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
            Self::Known(expression) => sorted_expression(expression).hash(state),
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

fn sorted_expression(expression: &Expression) -> String {
    expression.as_ref().split(' ').sorted().collect()
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
    fn licenses_with_parentheses_are_not_equal() {
        assert_ne!(
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
}
