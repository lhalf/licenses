use std::collections::HashMap;

#[allow(unused)]
struct Config {
    skipped: HashMap<String, Vec<String>>,
}

#[allow(unused)]
fn parse_config(contents: String) -> anyhow::Result<Config> {
    Err(anyhow::anyhow!(""))
}

#[cfg(test)]
mod tests {
    use crate::config::parse_config;

    #[test]
    fn empty_config_is_invalid() {
        assert!(parse_config(String::new()).is_err());
    }
}
