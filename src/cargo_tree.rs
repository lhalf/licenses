use anyhow::Context;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::process::Command;

use crate::dependency::Dependency;

pub fn get_dependencies(depth: u8) -> Result<BTreeSet<Dependency>, anyhow::Error> {
    let output = Command::new("cargo")
        .args(args(depth))
        .output()
        .context("failed to call cargo tree")?;

    parse_dependencies(
        &String::from_utf8(output.stdout).context("cargo tree output contained invalid utf8")?,
    )
}

fn args(depth: u8) -> Vec<String> {
    format!(
        "tree --depth {} --format {{p}} --prefix none --no-dedupe",
        depth
    )
    .split_whitespace()
    .map(String::from)
    .collect()
}

fn parse_dependencies(input: &str) -> Result<BTreeSet<Dependency>, anyhow::Error> {
    input
        .split('\n')
        .skip(1) // removes own library name from output
        .unique()
        .filter(|dependency| !dependency.is_empty())
        .sorted()
        .map(Dependency::parse)
        .collect()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use crate::{cargo_tree::parse_dependencies, dependency::Dependency};

    #[test]
    fn invalid_version_in_dependency() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)
example v0.hehe.2
example2 v3.4.5";

        assert_eq!(
            "invalid version in dependency",
            parse_dependencies(cargo_tree_output)
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn invalid_name_in_dependency() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)
example v0.1.2
exam\\ple2 v3.4.5";

        assert_eq!(
            "invalid dependency name",
            parse_dependencies(cargo_tree_output)
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn empty() {
        let cargo_tree_output = "";

        assert_eq!(
            BTreeSet::new(),
            parse_dependencies(cargo_tree_output).unwrap()
        )
    }

    #[test]
    fn no_dependencies() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)";

        assert_eq!(
            BTreeSet::new(),
            parse_dependencies(cargo_tree_output).unwrap()
        )
    }

    #[test]
    fn duplicated_dependencies() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)
example v0.1.2
example v0.1.2";

        assert_eq!(
            BTreeSet::from([Dependency {
                name: "example".to_string(),
                version: "0.1.2".to_string()
            },]),
            parse_dependencies(cargo_tree_output).unwrap()
        )
    }

    #[test]
    fn empty_row_in_dependencies() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)
example v0.1.2

example2 v3.4.5";

        assert_eq!(
            BTreeSet::from([
                Dependency {
                    name: "example".to_string(),
                    version: "0.1.2".to_string()
                },
                Dependency {
                    name: "example2".to_string(),
                    version: "3.4.5".to_string()
                }
            ]),
            parse_dependencies(cargo_tree_output).unwrap()
        )
    }

    #[test]
    fn valid_dependencies() {
        let cargo_tree_output = "my-library v0.1.0 (/home/example/my-library)
example v0.1.2
example2 v3.4.5";

        assert_eq!(
            BTreeSet::from([
                Dependency {
                    name: "example".to_string(),
                    version: "0.1.2".to_string()
                },
                Dependency {
                    name: "example2".to_string(),
                    version: "3.4.5".to_string()
                }
            ]),
            parse_dependencies(cargo_tree_output).unwrap()
        )
    }
}
