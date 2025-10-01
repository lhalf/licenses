#![allow(unused)]

use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use std::collections::HashMap;
use std::path::PathBuf;

fn diff_licenses(
    path: PathBuf,
    file_io: &impl FileIO,
    all_licenses: HashMap<Package, Vec<DirEntry>>,
) -> anyhow::Result<Vec<String>> {
    let _ = file_io.read_dir(&path)?;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::diff_licenses;
    use crate::file_io::FileIOSpy;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn failure_to_read_licenses_directory_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        assert!(diff_licenses(PathBuf::new(), &file_io_spy, HashMap::new()).is_err());
    }
}
