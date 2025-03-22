#[derive(PartialEq, Debug)]
struct Dependency {
    name: String,
    version: String,
}

impl Dependency {
    pub fn parse(input: &str) -> Result<Self, anyhow::Error> {
        let (name, version) = input
            .split_once(" v")
            .ok_or_else(|| anyhow::anyhow!("no seperator found in dependency"))?;

        Ok(Dependency {
            name: validate_name(name)?,
            version: validate_version(version)?,
        })
    }
}

fn validate_version(input: &str) -> Result<String, anyhow::Error> {
    if input
        .split('.')
        .filter_map(|part| part.parse::<u8>().ok())
        .count()
        == 3
    {
        Ok(input.to_string())
    } else {
        Err(anyhow::anyhow!("invalid version in dependency"))
    }
}

fn validate_name(input: &str) -> Result<String, anyhow::Error> {
    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Ok(input.to_string());
    } else {
        return Err(anyhow::anyhow!("invalid dependency name"));
    }
}

#[cfg(test)]
mod tests {
    use crate::dependencies::Dependency;

    #[test]
    fn invalid_name() {
        assert_eq!(
            "invalid dependency name",
            Dependency::parse("inva'lid v0.1.2")
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn invalid_version_no_seperator() {
        assert_eq!(
            "no seperator found in dependency",
            Dependency::parse("example 0.1.2")
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn invalid_version_not_enough_parts() {
        assert_eq!(
            "invalid version in dependency",
            Dependency::parse("example v0.1")
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn invalid_version_invalid_part() {
        assert_eq!(
            "invalid version in dependency",
            Dependency::parse("example v0.1.invalid")
                .unwrap_err()
                .root_cause()
                .to_string()
        )
    }

    #[test]
    fn valid_dependency() {
        assert_eq!(
            Dependency {
                name: "example".to_string(),
                version: "0.1.2".to_string()
            },
            Dependency::parse("example v0.1.2").unwrap()
        )
    }
}
