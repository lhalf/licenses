use anyhow::Context;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::process::Command;

pub fn crate_names(
    depth: Option<u8>,
    include_dev_dependencies: bool,
    include_build_dependencies: bool,
    excluded_workspaces: Vec<String>,
) -> anyhow::Result<BTreeSet<String>> {
    to_crate_names(
        Command::new("cargo")
            .args(args(
                depth,
                include_dev_dependencies,
                include_build_dependencies,
                excluded_workspaces,
            ))
            .output()
            .context("failed to call cargo tree")?
            .stdout,
    )
}

fn args(
    depth: Option<u8>,
    include_dev_dependencies: bool,
    include_build_dependencies: bool,
    excluded_workspaces: Vec<String>,
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

    if let Some(depth) = depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }

    if !excluded_workspaces.is_empty() {
        args.push("--workspace".to_string());
        excluded_workspaces.into_iter().for_each(|workspace| {
            args.push("--exclude".to_string());
            args.push(workspace);
        });
    };

    args
}

fn to_crate_names(output: Vec<u8>) -> anyhow::Result<BTreeSet<String>> {
    Ok(String::from_utf8(output)
        .context("cargo tree output contained invalid UTF-8")?
        .replace(" ", "")
        .split('\n')
        .map(|package| package.to_string().replace("-", "_"))
        .filter(|package| !package.is_empty())
        .unique()
        .collect())
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
            args(None, false, false, vec![])
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
            args(None, true, false, vec![])
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
            args(None, false, true, vec![])
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
            args(Some(1), true, true, vec![])
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
            args(None, true, true, vec!["excluded".to_string()])
        );
    }

    #[test]
    fn invalid_utf8_in_cargo_tree_output() {
        assert_eq!(
            "cargo tree output contained invalid UTF-8",
            to_crate_names(vec![255]).unwrap_err().to_string()
        );
    }

    #[test]
    fn strips_whitespace_from_cargo_tree_output() {
        assert_eq!(
            BTreeSet::new(),
            to_crate_names(b"                 ".to_vec()).unwrap()
        );
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"       example".to_vec()).unwrap()
        );
    }

    #[test]
    fn ignores_empty_entries_in_cargo_tree_output() {
        assert_eq!(BTreeSet::new(), to_crate_names(b"\n\n\n".to_vec()).unwrap());
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"\nexample\n\n".to_vec()).unwrap()
        );
    }

    #[test]
    fn normalises_crate_names_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from(["example_one".to_string(), "example_two".to_string()]),
            to_crate_names(b"example-one\nexample_two".to_vec()).unwrap()
        );
    }

    #[test]
    fn only_returns_unique_crate_names_in_cargo_tree_output() {
        assert_eq!(
            BTreeSet::from(["example".to_string()]),
            to_crate_names(b"example\n   example    \n\nexample".to_vec()).unwrap()
        );
    }
}
