pub fn split_licenses(licenses: &str) -> Vec<&str> {
    let mut parts = vec![licenses];

    for seperator in ["OR", "AND", "/"] {
        parts = parts
            .into_iter()
            .flat_map(|license| license.split(seperator).map(str::trim))
            .collect();
    }

    parts
        .into_iter()
        .map(|s| s.trim_matches(&['(', ')'][..]))
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_single_license() {
        assert_eq!(vec!["MIT"], split_licenses("MIT"));
    }

    #[test]
    fn split_licenses_with_or() {
        assert_eq!(vec!["MIT", "Apache-2.0"], split_licenses("MIT OR Apache-2.0"));
        assert_eq!(vec!["MIT", "Apache-2.0"], split_licenses("MIT/Apache-2.0"));
    }

    #[test]
    fn split_licenses_with_and() {
        assert_eq!(vec!["MIT", "Apache-2.0"], split_licenses("MIT AND Apache-2.0"));
    }

    #[test]
    fn split_licenses_with_parenthesis() {
        assert_eq!(
            vec!["MIT", "Apache-2.0", "UNICODE"],
            split_licenses("(MIT OR Apache-2.0) AND UNICODE")
        );
    }

    #[test]
    fn split_licenses_with_with() {
        assert_eq!(
            vec!["GPL-2.0-or-later WITH Bison-exception-2.2", "MIT"],
            split_licenses("GPL-2.0-or-later WITH Bison-exception-2.2 AND MIT")
        );
    }

    #[test]
    fn split_licenses_with_nested_parentheses() {
        assert_eq!(
            vec!["MIT", "GPL-2.0-or-later WITH Bison-exception-2.2", "UNICODE"],
            split_licenses("(MIT OR (GPL-2.0-or-later WITH Bison-exception-2.2)) AND UNICODE")
        );
    }
}