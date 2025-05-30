use std::path::PathBuf;

pub trait FindLicenses {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error>;
}

trait DirEntry {}

fn find_licenses_in_directory(
    _dir_entries: impl Iterator<Item = impl DirEntry>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    Ok(Vec::new())
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
    use super::PathBuf;
    use super::{DirEntry, find_licenses_in_directory};
    pub struct DirEntryFake {}
    impl DirEntry for DirEntryFake {}
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
            find_licenses_in_directory([DirEntryFake{}].into_iter()).unwrap()
        )
    }
}
