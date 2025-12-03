use crate::SummaryArgs;
use crate::cargo_metadata::Package;
use crate::config::Config;
use crate::file_io::FileIO;
use crate::licenses::check::check_licenses;
use crate::licenses::collect::collect_licenses;
use crate::licenses::copy::copy_licenses;
use crate::licenses::diff::diff_licenses;
use crate::licenses::summarise::{crates_per_license, summarise};
use crate::log::progress_bar;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub fn collect(
    file_io: &impl FileIO,
    config: &Config,
    filtered_packages: &[Package],
    path: String,
) -> anyhow::Result<()> {
    let path = PathBuf::from(path);
    let progress_bar = progress_bar("collecting licenses");

    create_output_folder(&path)?;

    let all_licenses = collect_licenses(file_io, filtered_packages, &config.crate_configs)?;

    let statuses = check_licenses(file_io, progress_bar, &all_licenses, &config.crate_configs);

    if statuses.any_invalid() {
        print!("{statuses}");
    }

    copy_licenses(file_io, all_licenses, path, &config.crate_configs)?;
    Ok(())
}

pub fn summary(filtered_packages: Vec<Package>, args: SummaryArgs) -> anyhow::Result<()> {
    let crates_per_license = crates_per_license(filtered_packages);

    println!(
        "{}",
        // clap should make it impossible for both to be true
        if args.json {
            serde_json::to_string_pretty(&crates_per_license)?
        } else if args.toml {
            toml::to_string_pretty(&crates_per_license)?
        } else {
            summarise(crates_per_license)
        }
    );

    Ok(())
}

pub fn check(
    file_io: &impl FileIO,
    config: &Config,
    filtered_packages: &[Package],
) -> anyhow::Result<()> {
    let progress_bar = progress_bar("checking licenses");

    let statuses = check_licenses(
        file_io,
        progress_bar,
        &collect_licenses(file_io, filtered_packages, &config.crate_configs)?,
        &config.crate_configs,
    );

    if statuses.any_invalid() {
        print!("{statuses}");
        std::process::exit(1)
    }

    Ok(())
}

pub fn diff(
    file_io: &impl FileIO,
    config: &Config,
    filtered_packages: &[Package],
    path: String,
) -> anyhow::Result<()> {
    if let Ok(diff) = diff_licenses(
        file_io,
        PathBuf::from(path),
        &config.crate_configs,
        collect_licenses(file_io, filtered_packages, &config.crate_configs)?,
    ) && !diff.is_empty()
    {
        print!("{diff}");
        std::process::exit(1)
    }

    Ok(())
}

fn create_output_folder(path: &Path) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir(path).context("failed to create output folder")
}
