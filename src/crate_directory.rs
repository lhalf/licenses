use std::path::PathBuf;

pub trait CrateDirectory {
    fn get_licenses(&self) -> Option<Vec<PathBuf>>;
}

#[cfg(test)]
pub struct CrateDirectoryFake {
    licenses: Option<Vec<String>>,
}

#[cfg(test)]
impl CrateDirectoryFake {
    pub fn containing_licenses(licenses: Option<Vec<&str>>) -> Self {
        Self {
            licenses: licenses.map(|license| license.into_iter().map(String::from).collect()),
        }
    }
}

#[cfg(test)]
impl CrateDirectory for CrateDirectoryFake {
    fn get_licenses(&self) -> Option<Vec<PathBuf>> {
        self.licenses.as_ref().map(|licenses| {
            licenses.iter().map(PathBuf::from).collect()
        })
    }
}
