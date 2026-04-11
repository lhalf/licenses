use crate::GlobalArgs;
use crate::file_io::FileIO;
use crate::licenses::status::LicenseStatus;
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub global: GlobalArgs,
    #[serde(rename = "crates")]
    pub crate_configs: HashMap<String, CrateConfig>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct CrateConfig {
    pub skip: Vec<String>,
    pub allow: Option<LicenseStatus>,
    pub include: Vec<IncludedLicense>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(untagged)]
pub enum IncludedLicense {
    Text { name: String, text: String },
}

impl GlobalArgs {
    fn merge(&mut self, other: Self) {
        self.dev |= other.dev;
        self.build |= other.build;
        self.all_features |= other.all_features;
        self.no_default_features |= other.no_default_features;
        if other.depth.is_some() {
            self.depth = other.depth;
        }
        self.feature.extend(other.feature);
        self.exclude.extend(other.exclude);
        self.ignore.extend(other.ignore);
    }
}

pub fn load_config(file_io: &impl FileIO, mut global_args: GlobalArgs) -> anyhow::Result<Config> {
    if let Some(path) = global_args.config.take() {
        let mut config = parse_config(&file_io.read_file(&path)?)?;
        config.crate_configs = normalised_crate_names(config.crate_configs);
        config.global.merge(global_args);
        Ok(config)
    } else {
        Ok(Config {
            global: global_args,
            crate_configs: HashMap::new(),
        })
    }
}

fn parse_config(contents: &str) -> anyhow::Result<Config> {
    toml::from_str(contents).context("failed to parse config")
}

fn normalised_crate_names(crates: HashMap<String, CrateConfig>) -> HashMap<String, CrateConfig> {
    crates
        .into_iter()
        .map(|(crate_name, config)| (crate_name.replace('-', "_"), config))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::GlobalArgs;
    use crate::config::{Config, CrateConfig, IncludedLicense, load_config, parse_config};
    use crate::file_io::FileIOSpy;
    use crate::licenses::status::LicenseStatus;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn empty_config_is_valid() {
        assert_eq!(config_with_crates([]), parse_config("").unwrap());
    }

    #[test]
    fn config_with_invalid_heading_is_invalid() {
        let contents = r"[invalid]";
        assert!(parse_config(contents).is_err());
    }

    #[test]
    fn config_with_valid_heading_but_no_skipped_files_is_valid() {
        for contents in [
            r"[crates.anyhow]",
            r"[global]
        [crates.anyhow]",
        ] {
            assert_eq!(
                config_with_crates([("anyhow", crate_config(&[], &[], None))]),
                parse_config(contents).unwrap()
            );
        }
    }

    #[test]
    fn config_with_valid_heading_and_allowed_warning_is_valid() {
        let contents = r#"
        [crates]
        anyhow = { allow = "too few" }"#;
        assert_eq!(
            config_with_crates([(
                "anyhow",
                crate_config(&[], &[], Some(LicenseStatus::TooFew))
            )]),
            parse_config(contents).unwrap()
        );
    }

    #[test]
    fn config_with_invalid_key_pair_is_invalid() {
        for contents in [
            r#"[crates.anyhow]
            lemon = "cheese""#,
            r#"[global] 
            config = "not allowed""#,
        ] {
            assert!(parse_config(contents).is_err());
        }
    }

    #[test]
    fn config_with_valid_heading_and_single_skipped_file_is_valid() {
        for contents in [
            r#"[crates.anyhow]
            skip = ["COPYING"]"#,
            r#"[crates]
            anyhow = { skip = ["COPYING"]}"#,
        ] {
            assert_eq!(
                config_with_crates([("anyhow", crate_config(&["COPYING"], &[], None))]),
                parse_config(contents).unwrap()
            );
        }
    }

    #[test]
    fn config_with_valid_heading_and_included_text_license_is_valid() {
        for contents in [
            r#"[crates.anyhow]
            include = [{ name = "LICENSE", text = "some license text" }]"#,
            r#"[crates]
            anyhow = { include = [{ name = "LICENSE", text = "some license text" }]}"#,
        ] {
            assert_eq!(
                config_with_crates([(
                    "anyhow",
                    crate_config(
                        &[],
                        &[IncludedLicense::Text {
                            name: "LICENSE".to_string(),
                            text: "some license text".to_string()
                        }],
                        None
                    )
                )]),
                parse_config(contents).unwrap()
            );
        }
    }

    #[test]
    fn config_with_multiple_valid_headings_and_multiple_skipped_files() {
        let contents = r#"
        [crates.anyhow]
        skip = ["COPYING"]
        [crates.another]
        skip = ["LICENSE-WRONG","COPYRIGHT"]"#;
        assert_eq!(
            config_with_crates([
                ("anyhow", crate_config(&["COPYING"], &[], None)),
                (
                    "another",
                    crate_config(&["LICENSE-WRONG", "COPYRIGHT"], &[], None)
                )
            ]),
            parse_config(contents).unwrap()
        );
    }

    #[test]
    fn config_with_comments_are_valid() {
        let contents = r#"
        [crates.anyhow]
        skip = ["COPYING"] # a comment"#;
        assert_eq!(
            config_with_crates([("anyhow", crate_config(&["COPYING"], &[], None))]),
            parse_config(contents).unwrap()
        );
    }

    #[test]
    fn config_with_duplicate_headings_are_invalid() {
        let contents = r#"
        [crates.anyhow]
        skip = ["COPYING"]
        [crates.anyhow]
        skip = ["LICENSE-WRONG","COPYRIGHT"]"#;
        assert!(parse_config(contents).is_err());
    }

    #[test]
    fn config_supports_all_global_args() {
        let contents = r#"
        [global]
        dev = true # a comment
        build = false
        depth = 1
        all-features = true
        no-default-features = true
        feature = ["feature"]
        exclude = ["test"]
        ignore = ["crate1","crate2"]"#;
        assert_eq!(
            Config {
                global: GlobalArgs {
                    dev: true,
                    build: false,
                    depth: Some(1),
                    all_features: true,
                    no_default_features: true,
                    feature: vec!["feature".to_string()],
                    exclude: vec!["test".to_string()],
                    ignore: vec!["crate1".to_string(), "crate2".to_string()],
                    config: None,
                },
                crate_configs: HashMap::new(),
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
            all_features: true,
            no_default_features: false,
            feature: vec!["feature1".to_string()],
            exclude: vec!["test".to_string()],
            ignore: vec![],
            config: None,
        };
        let global_args_2 = GlobalArgs {
            dev: false,
            build: true,
            depth: Some(20),
            all_features: false,
            no_default_features: true,
            feature: vec!["feature2".to_string()],
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
                all_features: true,
                no_default_features: true,
                feature: vec!["feature1".to_string(), "feature2".to_string()],
                exclude: vec!["test".to_string()],
                ignore: vec!["lemon".to_string()],
                config: None,
            },
            global_args_1
        );
    }

    #[test]
    fn errors_if_config_path_set_and_read_file_fails() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_file
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        let global_args = GlobalArgs {
            config: Some(PathBuf::from("path")),
            ..Default::default()
        };

        assert_eq!(
            "deliberate test error",
            load_config(&file_io_spy, global_args)
                .unwrap_err()
                .to_string()
        );
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

        let contents = r"
        [crates.normalise-me]";

        file_io_spy
            .read_file
            .returns
            .set([Ok(contents.to_string())]);

        assert_eq!(
            config_with_crates([("normalise_me", crate_config(&[], &[], None))]),
            load_config(
                &file_io_spy,
                GlobalArgs {
                    config: Some(PathBuf::from("path")),
                    ..Default::default()
                }
            )
            .unwrap()
        );
    }

    fn crate_config(
        skipped: &[&str],
        included: &[IncludedLicense],
        allow: Option<LicenseStatus>,
    ) -> CrateConfig {
        CrateConfig {
            skip: skipped.iter().map(ToString::to_string).collect(),
            allow,
            include: included.to_vec(),
        }
    }

    fn config_with_crates<I>(crates: I) -> Config
    where
        I: IntoIterator<Item = (&'static str, CrateConfig)>,
    {
        Config {
            global: GlobalArgs::default(),
            crate_configs: crates
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    #[test]
    fn config_with_all_allowed_statuses() {
        for (input, expected) in [
            (r#"allow = "empty""#, LicenseStatus::Empty),
            (r#"allow = "none declared""#, LicenseStatus::NoneDeclared),
            (r#"allow = "too few""#, LicenseStatus::TooFew),
        ] {
            let contents = format!("[crates.test]\n{input}");
            let config = parse_config(&contents).unwrap();
            assert_eq!(
                Some(&expected),
                config.crate_configs.get("test").unwrap().allow.as_ref()
            );
        }
    }

    #[test]
    fn merge_preserves_first_depth_when_second_is_none() {
        let mut args1 = GlobalArgs {
            depth: Some(5),
            ..Default::default()
        };
        let args2 = GlobalArgs {
            depth: None,
            ..Default::default()
        };
        args1.merge(args2);
        // When second has None depth, first keeps its value
        // Looking at merge: `if other.depth.is_some() { self.depth = other.depth; }`
        assert_eq!(Some(5), args1.depth);
    }

    #[test]
    fn merge_overwrites_depth_when_second_has_value() {
        let mut args1 = GlobalArgs {
            depth: Some(5),
            ..Default::default()
        };
        let args2 = GlobalArgs {
            depth: Some(10),
            ..Default::default()
        };
        args1.merge(args2);
        assert_eq!(Some(10), args1.depth);
    }

    #[test]
    fn merge_combines_features() {
        let mut args1 = GlobalArgs {
            feature: vec!["f1".to_string()],
            ..Default::default()
        };
        let args2 = GlobalArgs {
            feature: vec!["f2".to_string()],
            ..Default::default()
        };
        args1.merge(args2);
        assert_eq!(vec!["f1".to_string(), "f2".to_string()], args1.feature);
    }

    #[test]
    fn merge_booleans_are_or() {
        let mut args1 = GlobalArgs {
            dev: false,
            build: true,
            ..Default::default()
        };
        let args2 = GlobalArgs {
            dev: true,
            build: false,
            ..Default::default()
        };
        args1.merge(args2);
        assert!(args1.dev);
        assert!(args1.build);
    }

    #[test]
    fn config_with_multiple_included_licenses() {
        let contents = r#"
        [crates.example]
        include = [
            { name = "LICENSE-A", text = "text a" },
            { name = "LICENSE-B", text = "text b" }
        ]"#;
        let config = parse_config(contents).unwrap();
        assert_eq!(2, config.crate_configs["example"].include.len());
    }
}
