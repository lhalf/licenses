use crate::cargo_metadata::Package;
use crate::config::Config;
use crate::create_output_folder;
use crate::file_io::FileIO;
use crate::licenses::check::check_licenses;
use crate::licenses::collect::collect_licenses;
use crate::licenses::copy::copy_licenses;
use crate::log::progress_bar;
use std::path::PathBuf;

pub fn collect(
    file_io: &impl FileIO,
    config: &Config,
    filtered_packages: Vec<Package>,
    path: String,
) -> anyhow::Result<()> {
    let path = PathBuf::from(path);
    let progress_bar = progress_bar("collecting licenses");

    create_output_folder(&path)?;

    let all_licenses = collect_licenses(file_io, &filtered_packages, &config.crate_configs)?;

    let statuses = check_licenses(file_io, progress_bar, &all_licenses, &config.crate_configs);

    if statuses.any_invalid() {
        print!("{statuses}");
    }

    copy_licenses(file_io, all_licenses, path, &config.crate_configs)?;
    Ok(())
}
