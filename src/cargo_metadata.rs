use anyhow::Context;
use cargo_metadata::camino::Utf8PathBuf;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
}
