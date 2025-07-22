use anyhow::Context;
use itertools::Itertools;
use std::collections::BTreeSet;

pub fn to_crate_names(output: Vec<u8>) -> anyhow::Result<BTreeSet<String>> {
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
    use crate::cargo_tree::to_crate_names;
    use std::collections::BTreeSet;

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
