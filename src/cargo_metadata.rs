use anyhow::Context;
use cargo_metadata::camino::Utf8PathBuf;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq, Clone)]
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
            path: Default::default(),
            normalised_name: name.to_string(),
            url: None,
            license: None,
        }
    }
}

impl Package {
    fn try_from_metadata(package: cargo_metadata::Package) -> anyhow::Result<Self> {
        Ok(Self {
            normalised_name: package.name.to_string().replace("-", "_"),
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
    use cargo_metadata::camino::Utf8PathBuf;
    use cargo_util_schemas::manifest::PackageName;
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
        )
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
        )
    }

    #[test]
    fn packages_without_repository_sets_link_to_none() {
        assert!(
            Package::try_from_metadata(metadata_package())
                .unwrap()
                .url
                .is_none()
        )
    }

    #[test]
    fn packages_without_license_set_to_none() {
        assert!(
            Package::try_from_metadata(metadata_package())
                .unwrap()
                .license
                .is_none()
        )
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
        set.insert(package_2.clone());
        set.insert(package_3.clone());

        assert_eq!(set.len(), 2);
        assert!(set.contains(&package_1));
        assert!(set.contains(&package_3));
    }
}
