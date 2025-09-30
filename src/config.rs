use crate::GlobalArgs;
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub global: GlobalArgs,
    #[serde(rename = "crate")]
    _crate: HashMap<String, CrateConfig>,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct CrateConfig {
    pub skipped: Vec<String>,
}

impl GlobalArgs {
    fn merge(&mut self, global_args: GlobalArgs) {
        self.dev |= global_args.dev;
        self.build |= global_args.build;
        if global_args.depth.is_some() {
            self.depth = global_args.depth;
        }
        self.exclude.extend(global_args.exclude);
        self.ignore.extend(global_args.ignore);
    }
}

pub fn load_config(global_args: GlobalArgs) -> anyhow::Result<Config> {
    match global_args.config.clone() {
        Some(path) => {
            let mut config =
                parse_config(std::fs::read_to_string(path).context("failed to read config file")?)?;
            config.global.merge(global_args);
            Ok(config)
        }
        None => Ok(Config {
            global: global_args,
            _crate: HashMap::new(),
        }),
    }
}

fn parse_config(contents: String) -> anyhow::Result<Config> {
    toml::from_str(&contents).context("failed to parse config")
}

#[cfg(test)]
mod tests {
    use crate::GlobalArgs;
    use crate::config::{Config, CrateConfig, parse_config};
    use std::collections::HashMap;

    #[test]
    fn empty_config_is_valid() {
        assert_eq!(
            Config {
                global: Default::default(),
                _crate: HashMap::new(),
            },
            parse_config(String::new()).unwrap()
        );
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
                global: Default::default(),
                _crate: [("anyhow".to_string(), CrateConfig { skipped: vec![] })]
                    .into_iter()
                    .collect(),
            },
            parse_config(contents.to_string()).unwrap()
        );
        let contents = r#"
        [global]
        [crate.anyhow]"#;
        assert_eq!(
            Config {
                global: Default::default(),
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
        let contents = r#"
        [global]
        config = "not allowed""#;
        assert!(parse_config(contents.to_string()).is_err());
    }

    #[test]
    fn config_with_valid_heading_and_single_skipped_file_is_valid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]"#;
        assert_eq!(
            Config {
                global: Default::default(),
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
                global: Default::default(),
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
                global: Default::default(),
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

    #[test]
    fn config_supports_all_global_args() {
        let contents = r#"
        [global]
        dev = true # a comment
        build = false
        depth = 1
        exclude = ["test"]
        ignore = ["crate1","crate2"]"#;
        assert_eq!(
            Config {
                global: GlobalArgs {
                    dev: true,
                    build: false,
                    depth: Some(1),
                    exclude: vec!["test".to_string()],
                    ignore: vec!["crate1".to_string(), "crate2".to_string()],
                    config: None,
                },
                _crate: HashMap::new(),
            },
            parse_config(contents.to_string()).unwrap()
        );
    }

    #[test]
    fn global_args_merges_correctly() {
        let mut global_args_1 = GlobalArgs {
            dev: true,
            build: false,
            depth: Some(10),
            exclude: vec!["test".to_string()],
            ignore: vec![],
            config: None,
        };
        let global_args_2 = GlobalArgs {
            dev: false,
            build: true,
            depth: Some(20),
            exclude: vec![],
            ignore: vec!["lemon".to_string()],
            config: None,
        };
        global_args_1.merge(global_args_2);
        assert_eq!(
            GlobalArgs {
                dev: true,
                build: true,
                depth: Some(20),
                exclude: vec!["test".to_string()],
                ignore: vec!["lemon".to_string()],
                config: None,
            },
            global_args_1
        )
    }
}
