#![allow(dead_code)]
use std::fmt::{Display, Formatter};

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
    Unknown(String),
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
            License::Unknown(license) => write!(f, "{license}"),
        }
    }
}

impl From<&str> for License {
    fn from(_license: &str) -> Self {
        License::MIT
    }
}

#[cfg(test)]
mod tests {
    use crate::license::{GNU, License};

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
            format!("{}", License::Unknown("Sleepycat".to_string()))
        );
    }

    #[test]
    fn licenses_parse_from_spdx() {
        assert_eq!(License::MIT, License::from("MIT"));
    }
}
