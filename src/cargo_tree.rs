use std::collections::BTreeSet;
use anyhow::Context;

pub fn to_crate_names(output: Vec<u8>) -> anyhow::Result<BTreeSet<String>> {
    String::from_utf8(output).context("cargo tree output contained invalid UTF-8")?;
    Ok(BTreeSet::new())
}

#[cfg(test)]
mod tests {
    use crate::cargo_tree::to_crate_names;

    #[test]
    fn invalid_utf8_in_cargo_tree_output() {
        assert_eq!("cargo tree output contained invalid UTF-8", to_crate_names(vec![255]).unwrap_err().to_string());
    }
}