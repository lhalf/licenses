use anyhow::Context;
use cargo_metadata::camino::Utf8PathBuf;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq, Clone, PartialOrd, Ord)]
pub struct Package {
    pub normalised_name: String,
    pub path: Utf8PathBuf,
    pub url: Option<String>,
    pub license: Option<String>,
}

#[cfg(test)]
impl Package {
    pub fn called(name: &str) -> Self {
        Self {
            path: Utf8PathBuf::default(),
            normalised_name: name.to_string(),
            url: None,
            license: None,
        }
    }
}

impl Package {
    fn try_from_metadata(package: cargo_metadata::Package) -> anyhow::Result<Self> {
        Ok(Self {
            normalised_name: package.name.to_string().replace('-', "_"),
            path: package
                .manifest_path
                .parent()
                .context("could not get parent path from manifest path")?
                .to_path_buf(),
            url: package.repository,
            license: package.license,
        })
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.normalised_name == other.normalised_name && self.license == other.license
    }
}

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalised_name.hash(state);
        self.license.hash(state);
    }
}

pub fn filtered_packages(
    all_packages: Vec<Package>,
    crates_we_want: &BTreeSet<String>,
) -> Vec<Package> {
    all_packages
        .into_iter()
        .filter(|package| crates_we_want.contains(&package.normalised_name))
        .collect()
}

pub fn try_get_packages() -> anyhow::Result<Vec<Package>> {
    cargo_metadata::MetadataCommand::new()
        .exec()
        .context("failed to call cargo metadata")?
        .packages
        .into_iter()
        .map(Package::try_from_metadata)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Package;
    use cargo_metadata::PackageName;
    use cargo_metadata::camino::Utf8PathBuf;
    use std::collections::HashSet;
    use std::str::FromStr;

    fn metadata_package() -> cargo_metadata::Package {
        serde_json::from_str(
            r#"{
            "name": "example",
            "version": "0.0.0",
            "authors": [],
            "id": "dummy-id 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
            "source": null,
            "description": null,
            "dependencies": [],
            "license": null,
            "license_file": null,
            "targets": [],
            "features": {},
            "manifest_path": "some/path/Cargo.toml",
            "categories": [],
            "keywords": [],
            "readme": null,
            "repository": null,
            "homepage": null,
            "documentation": null,
            "edition": "2021",
            "metadata": null,
            "links": null,
            "publish": null,
            "default_run": null,
            "rust_version": null
        }"#,
        )
        .unwrap()
    }

    #[test]
    fn package_names_are_normalised() {
        let mut metadata_package = metadata_package();
        metadata_package.name = PackageName::from_str("normalised-name").unwrap();
        assert_eq!(
            "normalised_name",
            Package::try_from_metadata(metadata_package)
                .unwrap()
                .normalised_name
        );
    }

    #[test]
    fn packages_without_valid_manifest_path_parent_fail() {
        let mut metadata_package = metadata_package();
        metadata_package.manifest_path = Utf8PathBuf::from("/");
        assert_eq!(
            "could not get parent path from manifest path",
            Package::try_from_metadata(metadata_package)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn packages_without_repository_sets_link_to_none() {
        assert!(
            Package::try_from_metadata(metadata_package())
                .unwrap()
                .url
                .is_none()
        );
    }

    #[test]
    fn packages_without_license_set_to_none() {
        assert!(
            Package::try_from_metadata(metadata_package())
                .unwrap()
                .license
                .is_none()
        );
    }

    #[test]
    fn packages_with_same_name_and_license_are_equal() {
        assert_eq!(
            Package {
                normalised_name: "toml".to_string(),
                path: Utf8PathBuf::from("/some/path/1"),
                url: Some("https://github.com/toml-rs/toml".to_string()),
                license: Some("MIT".to_string()),
            },
            Package {
                normalised_name: "toml".to_string(),
                path: Utf8PathBuf::from("/some/path/2"),
                url: Some("https://github.com/toml-rs/toml".to_string()),
                license: Some("MIT".to_string()),
            }
        );
    }

    #[test]
    fn packages_with_same_name_different_license_are_not_equal() {
        assert_ne!(
            Package {
                normalised_name: "toml".to_string(),
                path: Utf8PathBuf::from("/some/path/1"),
                url: Some("https://github.com/toml-rs/toml".to_string()),
                license: Some("MIT".to_string()),
            },
            Package {
                normalised_name: "toml".to_string(),
                path: Utf8PathBuf::from("/some/path/2"),
                url: Some("https://github.com/toml-rs/toml".to_string()),
                license: Some("Apache-2.0".to_string()),
            }
        );
    }

    #[test]
    fn packages_are_hashed_based_on_name_and_license() {
        let package_1 = Package {
            normalised_name: "toml".to_string(),
            path: Utf8PathBuf::from("/some/path/1"),
            url: None,
            license: Some("MIT".to_string()),
        };
        let package_2 = Package {
            normalised_name: "toml".to_string(),
            path: Utf8PathBuf::from("/some/path/2"),
            url: None,
            license: Some("MIT".to_string()),
        };
        let package_3 = Package {
            normalised_name: "toml".to_string(),
            path: Utf8PathBuf::from("/some/path/3"),
            url: None,
            license: Some("Apache-2.0".to_string()),
        };

        let mut set = HashSet::new();
        set.insert(package_1.clone());
        set.insert(package_2);
        set.insert(package_3.clone());

        assert_eq!(set.len(), 2);
        assert!(set.contains(&package_1));
        assert!(set.contains(&package_3));
    }

    #[test]
    fn filtered_packages_returns_only_matching_crates() {
        use super::filtered_packages;
        use std::collections::BTreeSet;

        let packages = vec![
            Package::called("alpha"),
            Package::called("beta"),
            Package::called("gamma"),
        ];

        let crates_we_want: BTreeSet<String> =
            ["alpha", "gamma"].into_iter().map(String::from).collect();

        let result = filtered_packages(packages, &crates_we_want);
        assert_eq!(2, result.len());
        assert!(result.iter().any(|p| p.normalised_name == "alpha"));
        assert!(result.iter().any(|p| p.normalised_name == "gamma"));
    }

    #[test]
    fn filtered_packages_returns_empty_when_no_matches() {
        use super::filtered_packages;
        use std::collections::BTreeSet;

        let packages = vec![Package::called("alpha")];
        let crates_we_want: BTreeSet<String> = ["beta"].into_iter().map(String::from).collect();

        assert!(filtered_packages(packages, &crates_we_want).is_empty());
    }

    #[test]
    fn filtered_packages_returns_empty_for_empty_inputs() {
        use super::filtered_packages;
        use std::collections::BTreeSet;

        assert!(filtered_packages(vec![], &BTreeSet::new()).is_empty());
    }

    #[test]
    fn packages_with_repository_sets_url() {
        let mut metadata_package = metadata_package();
        metadata_package.repository = Some("https://github.com/example/repo".to_string());
        assert_eq!(
            Some("https://github.com/example/repo".to_string()),
            Package::try_from_metadata(metadata_package).unwrap().url
        );
    }

    #[test]
    fn packages_with_license_sets_license() {
        let mut metadata_package = metadata_package();
        metadata_package.license = Some("MIT OR Apache-2.0".to_string());
        assert_eq!(
            Some("MIT OR Apache-2.0".to_string()),
            Package::try_from_metadata(metadata_package)
                .unwrap()
                .license
        );
    }

    #[test]
    fn packages_are_ordered_by_name_then_license() {
        let a = Package {
            normalised_name: "alpha".to_string(),
            path: Utf8PathBuf::new(),
            url: None,
            license: Some("MIT".to_string()),
        };
        let b = Package {
            normalised_name: "beta".to_string(),
            path: Utf8PathBuf::new(),
            url: None,
            license: Some("MIT".to_string()),
        };
        assert!(a < b);
    }
}
