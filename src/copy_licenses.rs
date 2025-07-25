use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use anyhow::Context;
use colored::Colorize;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn copy_licenses(
    file_io: impl FileIO,
    crates: BTreeSet<String>,
    all_packages: Vec<Package>,
    output_folder: PathBuf,
) -> anyhow::Result<()> {
    for package in all_packages {
        if !crates.contains(&package.normalised_name) {
            continue;
        }

        let licenses: Vec<DirEntry> = file_io
            .read_dir(package.path.as_ref())?
            .into_iter()
            .filter(|dir_entry| {
                file_io.is_file(&dir_entry.path)
                    && dir_entry
                        .name
                        .to_string_lossy()
                        .to_lowercase()
                        .starts_with("license")
            })
            .collect();

        if licenses.is_empty() {
            println!(
                "{}: did not find any licenses for {} - try looking here: {}",
                "warning".yellow().bold(),
                package.normalised_name,
                package.url
            );
            continue;
        }

        for license in licenses {
            file_io.copy_file(
                &license.path,
                &output_folder.join(format!(
                    "{}-{}",
                    package.normalised_name,
                    license
                        .path
                        .file_name()
                        .context("license name contained invalid UTF-8")?
                        .to_string_lossy()
                )),
            )?
        }
    }

    Ok(())
}
