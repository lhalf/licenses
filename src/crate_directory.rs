use std::path::PathBuf;

pub trait CrateDirectory {
    fn get_license(&self) -> Option<PathBuf>;
}

#[cfg(test)]
pub struct CrateDirectoryFake {
    license: Option<String>,
}

#[cfg(test)]
impl CrateDirectoryFake {
    pub fn containing_license(license_name: Option<&str>) -> Self {
        Self {
            license: license_name.map(|license_name| license_name.to_string()),
        }
    }
}

#[cfg(test)]
impl CrateDirectory for CrateDirectoryFake {
    fn get_license(&self) -> Option<PathBuf> {
        self.license.as_ref().map(|license| PathBuf::from(license))
    }
}
