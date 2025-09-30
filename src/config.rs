#![allow(unused)]
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
    #[serde(rename = "crate")]
    _crate: HashMap<String, CrateConfig>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct CrateConfig {
    #[serde(default)]
    pub skipped: Vec<String>,
}

fn parse_config(contents: String) -> anyhow::Result<Config> {
    toml::from_str(&contents).context("failed to parse config")
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, CrateConfig, parse_config};

    #[test]
    fn empty_config_is_invalid() {
        assert!(parse_config(String::new()).is_err());
    }

    #[test]
    fn config_with_invalid_heading_is_invalid() {
        let contents = r#"
        [invalid]"#;
        assert!(parse_config(contents.to_string()).is_err());
    }

    #[test]
    fn config_with_valid_heading_but_no_skipped_files_is_valid() {
        let contents = r#"
        [crate.anyhow]"#;
        assert_eq!(
            Config {
                _crate: [("anyhow".to_string(), CrateConfig { skipped: vec![] })]
                    .into_iter()
                    .collect(),
            },
            parse_config(contents.to_string()).unwrap()
        );
    }

    #[test]
    fn config_with_invalid_key_pair_is_invalid() {
        let contents = r#"
        [crate.anyhow]
        lemon = "cheese""#;
        assert!(parse_config(contents.to_string()).is_err());
    }

    #[test]
    fn config_with_valid_heading_and_single_skipped_file_is_valid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]"#;
        assert_eq!(
            Config {
                _crate: [(
                    "anyhow".to_string(),
                    CrateConfig {
                        skipped: vec!["COPYING".to_string()]
                    }
                )]
                .into_iter()
                .collect(),
            },
            parse_config(contents.to_string()).unwrap()
        );
    }

    #[test]
    fn config_with_multiple_valid_headings_and_multiple_skipped_files() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]
        [crate.another]
        skipped = ["LICENSE-WRONG","COPYRIGHT"]"#;
        assert_eq!(
            Config {
                _crate: [
                    (
                        "anyhow".to_string(),
                        CrateConfig {
                            skipped: vec!["COPYING".to_string()]
                        }
                    ),
                    (
                        "another".to_string(),
                        CrateConfig {
                            skipped: vec!["LICENSE-WRONG".to_string(), "COPYRIGHT".to_string()]
                        }
                    )
                ]
                .into_iter()
                .collect(),
            },
            parse_config(contents.to_string()).unwrap()
        );
    }

    #[test]
    fn config_with_comments_are_valid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"] # a comment"#;
        assert_eq!(
            Config {
                _crate: [(
                    "anyhow".to_string(),
                    CrateConfig {
                        skipped: vec!["COPYING".to_string()]
                    }
                )]
                .into_iter()
                .collect(),
            },
            parse_config(contents.to_string()).unwrap()
        );
    }

    #[test]
    fn config_with_duplicate_headings_are_invalid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]
        [crate.anyhow]
        skipped = ["LICENSE-WRONG","COPYRIGHT"]"#;
        assert!(parse_config(contents.to_string()).is_err());
    }
}
