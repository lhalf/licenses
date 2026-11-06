use crate::config::Config;
use anyhow::Context;
use std::collections::BTreeSet;
use std::process::Command;

pub fn package_keys(config: &Config) -> anyhow::Result<BTreeSet<String>> {
    to_package_keys(
        cargo_output_with_args(&args(config))?,
        config.global.ignore.as_slice(),
    )
}

fn to_package_keys(output: Vec<u8>, ignored_crates: &[String]) -> anyhow::Result<BTreeSet<String>> {
    String::from_utf8(output)
        .context("cargo tree output contained invalid UTF-8")?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_package_key(line, ignored_crates).transpose())
        .collect()
}

fn parse_package_key(line: &str, ignored_crates: &[String]) -> anyhow::Result<Option<String>> {
    let (crate_name, package_details) = line
        .split_once('|')
        .context("cargo tree output missing package details")?;

    let crate_name = crate_name.trim().replace('-', "_");
    if crate_name.is_empty() || ignored_crates.contains(&crate_name) {
        return Ok(None);
    }

    let version = package_details
        .split_whitespace()
        .nth(1)
        .and_then(|version| version.strip_prefix('v'))
        .filter(|version| !version.is_empty())
        .context("cargo tree output missing package version")?;

    Ok(Some(format!("{crate_name}@{version}")))
}

fn cargo_output_with_args(args: &[String]) -> anyhow::Result<Vec<u8>> {
    let output = Command::new("cargo")
        .args(args)
        .output()
        .context("failed to call cargo tree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr.strip_prefix("error: ").unwrap_or(&stderr).trim();
        return Err(anyhow::anyhow!("{message}"));
    }

    Ok(output.stdout)
}

fn args(config: &Config) -> Vec<String> {
    let mut args = vec![
        "tree".to_string(),
        "--format".to_string(),
        "{lib}|{p}".to_string(),
        "--prefix".to_string(),
        "none".to_string(),
        "--no-dedupe".to_string(),
    ];

    let mut edges = Vec::new();
    if !config.global.dev {
        edges.push("no-dev");
    }
    if !config.global.build {
        edges.push("no-build");
    }
    if !edges.is_empty() {
        args.push("--edges".to_string());
        args.push(edges.join(","));
    }

    if config.global.all_features {
        args.push("--all-features".to_string());
    }

    if config.global.no_default_features {
        args.push("--no-default-features".to_string());
    }

    if !config.global.feature.is_empty() {
        args.push("--features".to_string());
        args.push(config.global.feature.join(","));
    }

    if let Some(depth) = config.global.depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }

    if !config.global.exclude.is_empty() {
        args.push("--workspace".to_string());
        for workspace in &config.global.exclude {
            args.push("--exclude".to_string());
            args.push(workspace.clone());
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use crate::cargo_tree::{args, to_package_keys};
    use crate::config::Config;
    use std::collections::BTreeSet;

    #[test]
    fn default_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
            ],
            args(&Config::default())
        );
    }

    #[test]
    fn include_dev_dependencies_args() {
        let mut config = Config::default();
        config.global.dev = true;

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-build".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn include_build_dependencies_args() {
        let mut config = Config::default();
        config.global.build = true;

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn all_features_args() {
        let mut config = Config::default();
        config.global.all_features = true;

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--all-features".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn no_default_features_args() {
        let mut config = Config::default();
        config.global.no_default_features = true;

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--no-default-features".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn depth_1_args() {
        let mut config = Config::default();
        config.global.depth = Some(1);

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--depth".to_string(),
                "1".to_string()
            ],
            args(&config)
        );
    }

    #[test]
    fn excludes_specific_workspace_args() {
        let mut config = Config::default();
        config.global.exclude = vec!["excluded".to_string()];

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--workspace".to_string(),
                "--exclude".to_string(),
                "excluded".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn enable_specific_features_args() {
        let mut config = Config::default();
        config.global.feature = vec!["feature1".to_string(), "feature2".to_string()];

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--features".to_string(),
                "feature1,feature2".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn invalid_utf8_in_cargo_tree_output() {
        assert_eq!(
            "cargo tree output contained invalid UTF-8",
            to_package_keys(vec![255], &[]).unwrap_err().to_string()
        );
    }

    #[test]
    fn strips_whitespace_from_cargo_tree_output() {
        assert_eq!(
            BTreeSet::new(),
            to_package_keys(b"                 ".to_vec(), &[]).unwrap()
        );
        assert_eq!(
            BTreeSet::from(["example@1.0.0".to_string()]),
            to_package_keys(b"  example | example   v1.0.0 \r\n".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn ignores_empty_entries_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::new(),
            to_package_keys(b"\n\n\n".to_vec(), &[]).unwrap()
        );
        assert_eq!(
            BTreeSet::from(["example@1.0.0".to_string()]),
            to_package_keys(b"\nexample|example v1.0.0\n\n".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn normalises_crate_names_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from([
                "example_one@1.0.0".to_string(),
                "example_two@2.0.0".to_string()
            ]),
            to_package_keys(
                b"example-one|example-one v1.0.0\nexample_two|example-two v2.0.0".to_vec(),
                &[],
            )
            .unwrap()
        );
    }

    #[test]
    fn only_returns_unique_package_keys_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from(["example@1.0.0".to_string(), "example@2.0.0".to_string()]),
            to_package_keys(
                b"example|example v1.0.0\n example|example v1.0.0 \n\nexample|example v2.0.0"
                    .to_vec(),
                &[],
            )
            .unwrap()
        );
    }

    #[test]
    fn ignores_single_specified_crate() {
        assert_eq!(
            BTreeSet::from(["one@1.0.0".to_string()]),
            to_package_keys(
                b"one|one v1.0.0\nignore_two|ignore-two v1.0.0\nignore_two|ignore-two v2.0.0"
                    .to_vec(),
                &["ignore_two".to_string()],
            )
            .unwrap()
        );
    }

    #[test]
    fn ignores_multiple_specified_crates() {
        assert_eq!(
            BTreeSet::from(["one@1.0.0".to_string()]),
            to_package_keys(
                b"one|one v1.0.0\nignore_two|ignore-two v1.0.0\nignore_three|ignore-three v1.0.0"
                    .to_vec(),
                &["ignore_two".to_string(), "ignore_three".to_string()]
            )
            .unwrap()
        );
    }

    #[test]
    fn ignores_invalid_ignored_crates() {
        assert_eq!(
            BTreeSet::from([
                "one@1.0.0".to_string(),
                "two@1.0.0".to_string(),
                "three@1.0.0".to_string(),
            ]),
            to_package_keys(
                b"one|one v1.0.0\ntwo|two v1.0.0\nthree|three v1.0.0".to_vec(),
                &["four".to_string()],
            )
            .unwrap()
        );
    }

    #[test]
    fn include_both_dev_and_build_dependencies_args() {
        let mut config = Config::default();
        config.global.dev = true;
        config.global.build = true;

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn multiple_excludes_args() {
        let mut config = Config::default();
        config.global.exclude = vec!["workspace_a".to_string(), "workspace_b".to_string()];

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--workspace".to_string(),
                "--exclude".to_string(),
                "workspace_a".to_string(),
                "--exclude".to_string(),
                "workspace_b".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn combined_flags_args() {
        let mut config = Config::default();
        config.global.dev = true;
        config.global.all_features = true;
        config.global.depth = Some(3);

        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}|{p}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-build".to_string(),
                "--all-features".to_string(),
                "--depth".to_string(),
                "3".to_string(),
            ],
            args(&config)
        );
    }

    #[test]
    fn preserves_prerelease_and_build_versions_with_source_paths() {
        assert_eq!(
            BTreeSet::from(["example_one@1.2.3-rc.1+build-4".to_string()]),
            to_package_keys(
                b"example-one|example-one v1.2.3-rc.1+build-4 (/some path/example-one)".to_vec(),
                &[],
            )
            .unwrap()
        );
    }

    #[test]
    fn ignores_packages_without_libraries() {
        assert_eq!(
            BTreeSet::new(),
            to_package_keys(b"|binary v1.0.0 (/some/path)".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn missing_package_details_returns_error() {
        assert_eq!(
            "cargo tree output missing package details",
            to_package_keys(b"example".to_vec(), &[])
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn missing_package_version_returns_error() {
        for output in [
            "example|example",
            "example|example 1.0.0",
            "example|example v",
        ] {
            assert_eq!(
                "cargo tree output missing package version",
                to_package_keys(output.as_bytes().to_vec(), &[])
                    .unwrap_err()
                    .to_string()
            );
        }
    }
}
