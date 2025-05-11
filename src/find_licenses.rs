use std::path::PathBuf;

pub trait FindLicenses {
    fn find_licenses(&self) -> Result<Vec<PathBuf>, anyhow::Error>;
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
