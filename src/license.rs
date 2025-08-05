#![allow(dead_code)]
use std::fmt::{Display, Formatter};
use std::str::FromStr;

type Version = f32;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
enum License {
    MIT,
    Apache(Version),
    Unicode,
    Unlicense,
    LGPL(GNU),
    GPL(GNU),
    AGPL(Version),
    // missing MPL-2.0-no-copyleft-exception
    MPL(Version),
    BSL,
    Other(String),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
enum GNU {
    Only(Version),
    OrLater(Version),
}

impl Display for License {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            License::MIT => write!(f, "MIT"),
            License::Apache(version) => write!(f, "Apache-{version:.1}"),
            License::Unicode => write!(f, "Unicode-3.0"),
            License::Unlicense => write!(f, "Unlicense"),
            License::LGPL(GNU::Only(version)) => write!(f, "LGPL-{version:.1}-only"),
            License::LGPL(GNU::OrLater(version)) => write!(f, "LGPL-{version:.1}-or-later"),
            License::GPL(GNU::Only(version)) => write!(f, "GPL-{version:.1}-only"),
            License::GPL(GNU::OrLater(version)) => write!(f, "GPL-{version:.1}-or-later"),
            License::AGPL(version) => write!(f, "AGPL-{version:.1}"),
            License::MPL(version) => write!(f, "MPL-{version:.1}"),
            License::BSL => write!(f, "BSL-1.0"),
            License::Other(license) => write!(f, "{license}"),
        }
    }
}

impl FromStr for License {
    type Err = anyhow::Error;

    fn from_str(license: &str) -> anyhow::Result<Self> {
        if license.to_uppercase().contains(" WITH ") {
            return Ok(License::Other(license.to_string()));
        }

        if license == "MIT" {
            return Ok(License::MIT);
        }
        if license == "Unicode-3.0" {
            return Ok(License::Unicode);
        }
        if license == "Unlicense" {
            return Ok(License::Unlicense);
        }
        if license == "BSL-1.0" {
            return Ok(License::BSL);
        }

        if let Some(version) = license.strip_prefix("Apache-") {
            return Ok(License::Apache(version.parse()?));
        }

        if let Some(version) = license.strip_prefix("AGPL-") {
            return Ok(License::AGPL(version.parse()?));
        }

        if let Some(version) = license.strip_prefix("MPL-") {
            return Ok(License::MPL(version.parse()?));
        }

        if let Some(suffix) = license.strip_prefix("GPL-") {
            return parse_gnu_license(suffix).map(License::GPL);
        }

        if let Some(suffix) = license.strip_prefix("LGPL-") {
            return parse_gnu_license(suffix).map(License::LGPL);
        }

        Ok(License::Other(license.to_string()))
    }
}

fn parse_gnu_license(suffix: &str) -> anyhow::Result<GNU> {
    if let Some(version) = suffix.strip_suffix("-only") {
        Ok(GNU::Only(version.parse()?))
    } else if let Some(version) = suffix.strip_suffix("-or-later") {
        Ok(GNU::OrLater(version.parse()?))
    } else if let Some(version) = suffix.strip_suffix("+") {
        Ok(GNU::OrLater(version.parse()?))
    } else {
        Ok(GNU::Only(suffix.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use crate::license::{GNU, License};
    use std::str::FromStr;

    #[test]
    fn licenses_display_as_spdx() {
        assert_eq!("MIT", format!("{}", License::MIT));
        assert_eq!("Apache-2.0", format!("{}", License::Apache(2.0)));
        assert_eq!("Unicode-3.0", format!("{}", License::Unicode));
        assert_eq!("Unlicense", format!("{}", License::Unlicense));
        assert_eq!(
            "LGPL-3.0-only",
            format!("{}", License::LGPL(GNU::Only(3.0)))
        );
        assert_eq!(
            "LGPL-2.1-or-later",
            format!("{}", License::LGPL(GNU::OrLater(2.1)))
        );
        assert_eq!("GPL-2.0-only", format!("{}", License::GPL(GNU::Only(2.0))));
        assert_eq!(
            "GPL-1.0-or-later",
            format!("{}", License::GPL(GNU::OrLater(1.0)))
        );
        assert_eq!("AGPL-1.0", format!("{}", License::AGPL(1.0)));
        assert_eq!("MPL-1.1", format!("{}", License::MPL(1.1)));
        assert_eq!("BSL-1.0", format!("{}", License::BSL));
        assert_eq!(
            "Sleepycat",
            format!("{}", License::Other("Sleepycat".to_string()))
        );
    }

    #[test]
    fn licenses_parse_from_spdx() {
        assert_eq!(License::MIT, License::from_str("MIT").unwrap());
        assert_eq!(License::Unicode, License::from_str("Unicode-3.0").unwrap());
        assert_eq!(License::Unlicense, License::from_str("Unlicense").unwrap());
        assert_eq!(License::BSL, License::from_str("BSL-1.0").unwrap());
        assert_eq!(
            License::Apache(2.0),
            License::from_str("Apache-2.0").unwrap()
        );
        assert_eq!(License::AGPL(1.0), License::from_str("AGPL-1.0").unwrap());
        assert_eq!(License::MPL(1.1), License::from_str("MPL-1.1").unwrap());
        assert_eq!(
            License::GPL(GNU::Only(2.0)),
            License::from_str("GPL-2.0-only").unwrap()
        );
        assert_eq!(
            License::GPL(GNU::Only(2.0)),
            License::from_str("GPL-2.0").unwrap()
        );
        assert_eq!(
            License::GPL(GNU::OrLater(2.0)),
            License::from_str("GPL-2.0-or-later").unwrap()
        );
        assert_eq!(
            License::GPL(GNU::OrLater(2.0)),
            License::from_str("GPL-2.0+").unwrap()
        );
        assert_eq!(
            License::LGPL(GNU::Only(2.0)),
            License::from_str("LGPL-2.0-only").unwrap()
        );
        assert_eq!(
            License::LGPL(GNU::Only(2.0)),
            License::from_str("LGPL-2.0").unwrap()
        );
        assert_eq!(
            License::LGPL(GNU::OrLater(2.0)),
            License::from_str("LGPL-2.0-or-later").unwrap()
        );
        assert_eq!(
            License::LGPL(GNU::OrLater(2.0)),
            License::from_str("LGPL-2.0+").unwrap()
        );
        assert_eq!(
            License::Other("Sleepycat".to_string()),
            License::from_str("Sleepycat").unwrap()
        );
    }

    #[test]
    fn handles_licenses_with_with_clause() {
        assert_eq!(
            License::Other("GPL-2.0-or-later WITH Bison-exception-2.2".to_string()),
            License::from_str("GPL-2.0-or-later WITH Bison-exception-2.2").unwrap()
        );
    }
}
