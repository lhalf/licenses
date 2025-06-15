use std::path::PathBuf;

pub trait FindLicenses {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error>;
}

fn find_licenses_in_directory(dir_paths: impl Iterator<Item = PathBuf>) -> Vec<PathBuf> {
    dir_paths
        .filter(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .contains("license")
        })
        .collect()
}

#[cfg(test)]
pub struct CrateDirectoryFake {
    licenses: Vec<String>,
    fails_to_open: bool,
}

#[cfg(test)]
impl CrateDirectoryFake {
    pub fn containing_licenses(licenses: Vec<&str>) -> Self {
        Self {
            licenses: licenses.into_iter().map(String::from).collect(),
            fails_to_open: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            licenses: Vec::new(),
            fails_to_open: true,
        }
    }
}

#[cfg(test)]
impl FindLicenses for CrateDirectoryFake {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error> {
        if self.fails_to_open {
            return Err(anyhow::anyhow!("deliberate test error"));
        }

        Ok(self.licenses.iter().map(PathBuf::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_crate_directory_has_no_licences() {
        assert_eq!(
            Vec::<PathBuf>::new(),
            find_licenses_in_directory(Vec::new().into_iter())
        )
    }

    #[test]
    fn crate_directory_with_no_licences() {
        assert_eq!(
            Vec::<PathBuf>::new(),
            find_licenses_in_directory([PathBuf::new()].into_iter())
        )
    }

    #[test]
    fn crate_directory_with_one_licences_returns_path_for_licence() {
        assert_eq!(
            vec![PathBuf::from("path/license")],
            find_licenses_in_directory([PathBuf::from("path/license")].into_iter())
        )
    }

    #[test]
    fn crate_directory_with_multiple_licences_returns_path_for_licences() {
        assert_eq!(
            vec![
                PathBuf::from("path/license_1"),
                PathBuf::from("path/license_2")
            ],
            find_licenses_in_directory(
                [
                    PathBuf::from("path/license_1"),
                    PathBuf::from("path/license_2")
                ]
                .into_iter()
            )
        )
    }
}
