use crate::cargo_metadata::Package;
use colored::Colorize;
use itertools::Itertools;
use std::collections::HashMap;

pub fn crates_per_license(filtered_packages: Vec<Package>) -> HashMap<String, Vec<String>> {
    filtered_packages
        .into_iter()
        .filter_map(|package| {
            package
                .license
                .map(|license| (license, package.normalised_name))
        })
        .into_group_map()
}

pub fn summarise(crates_per_license: HashMap<String, Vec<String>>) -> String {
    crates_per_license
        .into_iter()
        .map(|(license, mut normalised_names)| {
            normalised_names.sort();
            format!(
                "{}: {}",
                license.bold(),
                normalised_names.join(",").dimmed()
            )
        })
        .sorted()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::summarise::{crates_per_license, summarise};
    use colored::Colorize;

    #[test]
    fn no_packages() {
        assert!(summarise(crates_per_license(Vec::new())).is_empty())
    }

    #[test]
    fn single_package_with_no_license() {
        assert!(
            summarise(crates_per_license(vec![Package {
                normalised_name: "no_license".to_string(),
                path: Default::default(),
                url: None,
                license: None,
            }]))
            .is_empty()
        )
    }

    #[test]
    fn single_package() {
        assert_eq!(
            format!("{}: {}", "MIT".bold(), "example".dimmed()),
            summarise(crates_per_license(vec![Package {
                normalised_name: "example".to_string(),
                path: Default::default(),
                url: None,
                license: Some("MIT".to_string()),
            }]))
        )
    }

    #[test]
    fn multiple_different_license_packages() {
        assert_eq!(
            format!(
                "{}: {}\n{}: {}",
                "Apache-2.0".bold(),
                "another".dimmed(),
                "MIT".bold(),
                "example".dimmed()
            ),
            summarise(crates_per_license(vec![
                Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("MIT".to_string()),
                },
                Package {
                    normalised_name: "another".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("Apache-2.0".to_string()),
                }
            ]))
        )
    }

    #[test]
    fn multiple_same_license_packages() {
        assert_eq!(
            format!("{}: {}", "MIT".bold(), "a,b,c".dimmed()),
            summarise(crates_per_license(vec![
                Package {
                    normalised_name: "c".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("MIT".to_string()),
                },
                Package {
                    normalised_name: "a".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("MIT".to_string()),
                },
                Package {
                    normalised_name: "b".to_string(),
                    path: Default::default(),
                    url: None,
                    license: Some("MIT".to_string()),
                }
            ]))
        )
    }
}
