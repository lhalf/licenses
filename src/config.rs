use crate::GlobalArgs;
use crate::file_io::FileIO;
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub global: GlobalArgs,
    #[serde(rename = "crate")]
    pub crates: HashMap<String, CrateConfig>,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct CrateConfig {
    pub skipped: Vec<String>,
}

impl GlobalArgs {
    fn merge(&mut self, other: GlobalArgs) {
        self.dev |= other.dev;
        self.build |= other.build;
        if other.depth.is_some() {
            self.depth = other.depth;
        }
        self.exclude.extend(other.exclude);
        self.ignore.extend(other.ignore);
    }
}

pub fn load_config(file_io: &impl FileIO, mut global_args: GlobalArgs) -> anyhow::Result<Config> {
    if let Some(path) = global_args.config.take() {
        let mut config = parse_config(&file_io.read_file(path.as_ref())?)?;
        config.crates = normalised_crate_names(config.crates);
        config.global.merge(global_args);
        Ok(config)
    } else {
        Ok(Config {
            global: global_args,
            crates: HashMap::new(),
        })
    }
}

fn parse_config(contents: &str) -> anyhow::Result<Config> {
    toml::from_str(contents).context("failed to parse config")
}

fn normalised_crate_names(crates: HashMap<String, CrateConfig>) -> HashMap<String, CrateConfig> {
    crates
        .into_iter()
        .map(|(crate_name, config)| (crate_name.replace("-", "_"), config))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::GlobalArgs;
    use crate::config::{Config, CrateConfig, load_config, parse_config};
    use crate::file_io::FileIOSpy;
    use std::collections::HashMap;

    #[test]
    fn empty_config_is_valid() {
        assert_eq!(
            Config {
                global: Default::default(),
                crates: HashMap::new(),
            },
            parse_config("").unwrap()
        );
    }

    #[test]
    fn config_with_invalid_heading_is_invalid() {
        let contents = r#"
        [invalid]"#;
        assert!(parse_config(contents).is_err());
    }

    #[test]
    fn config_with_valid_heading_but_no_skipped_files_is_valid() {
        let contents = r#"
        [crate.anyhow]"#;
        assert_eq!(
            Config {
                global: Default::default(),
                crates: [("anyhow".to_string(), CrateConfig { skipped: vec![] })]
                    .into_iter()
                    .collect(),
            },
            parse_config(contents).unwrap()
        );
        let contents = r#"
        [global]
        [crate.anyhow]"#;
        assert_eq!(
            Config {
                global: Default::default(),
                crates: [("anyhow".to_string(), CrateConfig { skipped: vec![] })]
                    .into_iter()
                    .collect(),
            },
            parse_config(contents).unwrap()
        );
    }

    #[test]
    fn config_with_invalid_key_pair_is_invalid() {
        let contents = r#"
        [crate.anyhow]
        lemon = "cheese""#;
        assert!(parse_config(contents).is_err());
        let contents = r#"
        [global]
        config = "not allowed""#;
        assert!(parse_config(contents).is_err());
    }

    #[test]
    fn config_with_valid_heading_and_single_skipped_file_is_valid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]"#;
        assert_eq!(
            Config {
                global: Default::default(),
                crates: [(
                    "anyhow".to_string(),
                    CrateConfig {
                        skipped: vec!["COPYING".to_string()]
                    }
                )]
                .into_iter()
                .collect(),
            },
            parse_config(contents).unwrap()
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
                crates: [
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
            parse_config(contents).unwrap()
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
                crates: [(
                    "anyhow".to_string(),
                    CrateConfig {
                        skipped: vec!["COPYING".to_string()]
                    }
                )]
                .into_iter()
                .collect(),
            },
            parse_config(contents).unwrap()
        );
    }

    #[test]
    fn config_with_duplicate_headings_are_invalid() {
        let contents = r#"
        [crate.anyhow]
        skipped = ["COPYING"]
        [crate.anyhow]
        skipped = ["LICENSE-WRONG","COPYRIGHT"]"#;
        assert!(parse_config(contents).is_err());
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
                crates: HashMap::new(),
            },
            parse_config(contents).unwrap()
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

    #[test]
    fn errors_if_config_path_set_and_read_file_fails() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        let global_args = GlobalArgs {
            dev: false,
            build: false,
            depth: None,
            exclude: vec![],
            ignore: vec![],
            config: Some("path".to_string()),
        };

        assert_eq!(
            "deliberate test error",
            load_config(&file_io_spy, global_args)
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn never_uses_file_io_if_config_path_not_set() {
        let file_io_spy = FileIOSpy::default();

        assert!(load_config(&file_io_spy, GlobalArgs::default()).is_ok());

        assert!(file_io_spy.read_file.arguments.take().is_empty());
    }

    #[test]
    fn crates_in_config_are_normalised() {
        let file_io_spy = FileIOSpy::default();

        let contents = r#"
        [crate.normalise-me]"#;

        file_io_spy
            .read_file
            .returns
            .set([Ok(contents.to_string())]);

        assert_eq!(
            Config {
                global: Default::default(),
                crates: [("normalise_me".to_string(), CrateConfig { skipped: vec![] })]
                    .into_iter()
                    .collect(),
            },
            load_config(
                &file_io_spy,
                GlobalArgs {
                    dev: false,
                    build: false,
                    depth: None,
                    exclude: vec![],
                    ignore: vec![],
                    config: Some("path".to_string()),
                }
            )
            .unwrap()
        );
    }
}
