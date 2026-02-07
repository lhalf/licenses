use crate::config::Config;
use anyhow::Context;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::process::Command;

pub fn crate_names(config: &Config) -> anyhow::Result<BTreeSet<String>> {
    to_crate_names(
        cargo_output_with_args(args(
            config.global.depth,
            config.global.dev,
            config.global.build,
            config.global.all_features,
            config.global.no_default_features,
            config.global.exclude.as_slice(),
        ))?,
        config.global.ignore.as_slice(),
    )
}

fn to_crate_names(output: Vec<u8>, ignored_crates: &[String]) -> anyhow::Result<BTreeSet<String>> {
    Ok(String::from_utf8(output)
        .context("cargo tree output contained invalid UTF-8")?
        .replace(' ', "")
        .split('\n')
        .map(|crate_name| crate_name.to_string().replace('-', "_"))
        .filter(|crate_name| !crate_name.is_empty() && !ignored_crates.contains(crate_name))
        .unique()
        .collect())
}

fn cargo_output_with_args(args: Vec<String>) -> anyhow::Result<Vec<u8>> {
    Ok(Command::new("cargo")
        .args(args)
        .output()
        .context("failed to call cargo tree")?
        .stdout)
}

fn args(
    depth: Option<u8>,
    include_dev_dependencies: bool,
    include_build_dependencies: bool,
    all_features: bool,
    no_default_features: bool,
    excluded_workspaces: &[String],
) -> Vec<String> {
    let mut args = vec![
        "tree".to_string(),
        "--format".to_string(),
        "{lib}".to_string(),
        "--prefix".to_string(),
        "none".to_string(),
        "--no-dedupe".to_string(),
    ];

    let mut edges = Vec::new();
    if !include_dev_dependencies {
        edges.push("no-dev");
    }
    if !include_build_dependencies {
        edges.push("no-build");
    }
    if !edges.is_empty() {
        args.push("--edges".to_string());
        args.push(edges.join(","));
    }

    if all_features {
        args.push("--all-features".to_string());
    }

    if no_default_features {
        args.push("--no-default-features".to_string());
    }

    if let Some(depth) = depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }

    if !excluded_workspaces.is_empty() {
        args.push("--workspace".to_string());
        for workspace in excluded_workspaces {
            args.push("--exclude".to_string());
            args.push(workspace.clone());
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use crate::cargo_tree::{args, to_crate_names};
    use std::collections::BTreeSet;

    #[test]
    fn default_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
            ],
            args(None, false, false, false, false, &[])
        );
    }

    #[test]
    fn include_dev_dependencies_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-build".to_string(),
            ],
            args(None, true, false, false, false, &[])
        );
    }

    #[test]
    fn include_build_dependencies_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev".to_string(),
            ],
            args(None, false, true, false, false, &[])
        );
    }

    #[test]
    fn all_features_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--all-features".to_string(),
            ],
            args(None, false, false, true, false, &[])
        );
    }

    #[test]
    fn no_default_features_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--edges".to_string(),
                "no-dev,no-build".to_string(),
                "--no-default-features".to_string(),
            ],
            args(None, false, false, false, true, &[])
        );
    }

    #[test]
    fn depth_1_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--depth".to_string(),
                "1".to_string()
            ],
            args(Some(1), true, true, false, false, &[])
        );
    }

    #[test]
    fn excludes_specific_workspace_args() {
        assert_eq!(
            vec![
                "tree".to_string(),
                "--format".to_string(),
                "{lib}".to_string(),
                "--prefix".to_string(),
                "none".to_string(),
                "--no-dedupe".to_string(),
                "--workspace".to_string(),
                "--exclude".to_string(),
                "excluded".to_string(),
            ],
            args(None, true, true, false, false, &["excluded".to_string()])
        );
    }

    #[test]
    fn invalid_utf8_in_cargo_tree_output() {
        assert_eq!(
            "cargo tree output contained invalid UTF-8",
            to_crate_names(vec![255], &[]).unwrap_err().to_string()
        );
    }

    #[test]
    fn strips_whitespace_from_cargo_tree_output() {
        assert_eq!(
            BTreeSet::new(),
            to_crate_names(b"                 ".to_vec(), &[]).unwrap()
        );
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"       example".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn ignores_empty_entries_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::new(),
            to_crate_names(b"\n\n\n".to_vec(), &[]).unwrap()
        );
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"\nexample\n\n".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn normalises_crate_names_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from(["example_one".to_string(), "example_two".to_string()]),
            to_crate_names(b"example-one\nexample_two".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn only_returns_unique_crate_names_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"example\n   example    \n\nexample".to_vec(), &[]).unwrap()
        );
    }

    #[test]
    fn ignores_single_specified_crate() {
        assert_eq!(
            BTreeSet::from(["one".to_string()]),
            to_crate_names(b"one\nignore_two".to_vec(), &["ignore_two".to_string()]).unwrap()
        );
    }

    #[test]
    fn ignores_multiple_specified_crates() {
        assert_eq!(
            BTreeSet::from(["one".to_string()]),
            to_crate_names(
                b"one\nignore_two\nignore_three".to_vec(),
                &["ignore_two".to_string(), "ignore_three".to_string()]
            )
            .unwrap()
        );
    }

    #[test]
    fn ignores_invalid_ignored_crates() {
        assert_eq!(
            BTreeSet::from(["one".to_string(), "two".to_string(), "three".to_string()]),
            to_crate_names(b"one\ntwo\nthree".to_vec(), &["four".to_string()]).unwrap()
        );
    }
}
