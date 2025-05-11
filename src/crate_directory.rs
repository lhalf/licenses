use std::path::PathBuf;

pub trait GetLicenses {
    fn get_licenses(&self) -> Result<Option<Vec<PathBuf>>, anyhow::Error>;
}

#[cfg(test)]
pub struct CrateDirectoryFake {
    licenses: Option<Vec<String>>,
    fails: bool,
}

#[cfg(test)]
impl CrateDirectoryFake {
    pub fn containing_licenses(licenses: Option<Vec<&str>>) -> Self {
        Self {
            licenses: licenses.map(|license| license.into_iter().map(String::from).collect()),
            fails: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            licenses: None,
            fails: true,
        }
    }
}

#[cfg(test)]
impl GetLicenses for CrateDirectoryFake {
    fn get_licenses(&self) -> Result<Option<Vec<PathBuf>>, anyhow::Error> {
        if self.fails {
            return Err(anyhow::anyhow!("deliberate test error"));
        }

        Ok(self
            .licenses
            .as_ref()
            .map(|licenses| licenses.iter().map(PathBuf::from).collect()))
    }
}
