use std::path::PathBuf;
use crate::dir_entry::DirEntry;

pub trait FindLicenses {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error>;
}

fn find_licenses_in_directory(
    dir_entries: impl Iterator<Item = impl DirEntry>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    Ok(dir_entries
        .filter(|dir_entry| dir_entry.filename().contains("license"))
        .map(|dir_entry| dir_entry.path())
        .collect())
}

#[cfg(test)]
pub struct CrateDirectoryFake {
    licenses: Vec<String>,
    fails: bool,
}

#[cfg(test)]
impl CrateDirectoryFake {
    pub fn containing_licenses(licenses: Vec<&str>) -> Self {
        Self {
            licenses: licenses.into_iter().map(String::from).collect(),
            fails: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            licenses: Vec::new(),
            fails: true,
        }
    }
}

#[cfg(test)]
impl FindLicenses for CrateDirectoryFake {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error> {
        if self.fails {
            return Err(anyhow::anyhow!("deliberate test error"));
        }

        Ok(self.licenses.iter().map(PathBuf::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dir_entry::DirEntryFake;

    #[test]
    fn empty_crate_directory_has_no_licences() {
        assert_eq!(
            Vec::<PathBuf>::new(),
            find_licenses_in_directory(Vec::<DirEntryFake>::new().into_iter()).unwrap()
        )
    }

    #[test]
    fn crate_directory_with_no_licences() {
        assert_eq!(
            Vec::<PathBuf>::new(),
            find_licenses_in_directory(
                [DirEntryFake {
                    filename: "file.txt",
                    path: PathBuf::new()
                }]
                .into_iter()
            )
            .unwrap()
        )
    }

    #[test]
    fn crate_directory_with_one_licences_returns_path_for_licence() {
        assert_eq!(
            vec![PathBuf::from("path/license")],
            find_licenses_in_directory(
                [DirEntryFake {
                    filename: "license",
                    path: PathBuf::from("path/license")
                }]
                .into_iter()
            )
            .unwrap()
        )
    }
}
