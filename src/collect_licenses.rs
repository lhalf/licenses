use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::file_io::FileIO;
use crate::is_license::is_license;

pub fn collect_licenses(file_io: &impl FileIO, package: &Package) -> anyhow::Result<Vec<DirEntry>> {
    let licenses: Vec<DirEntry> = file_io
        .read_dir(package.path.as_ref())?
        .into_iter()
        .filter(is_license)
        .collect();
    Ok(licenses)
}
