use crate::cargo_metadata::{Package, try_get_packages};
use crate::cargo_tree::crate_names;
use crate::file_io::{DirEntry, FileIO, FileSystem};
use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use std::collections::BTreeSet;
use std::path::PathBuf;

mod cargo_metadata;
mod cargo_tree;
mod file_io;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The output license folder path
    #[arg(short, long, default_value_t = String::from("licenses"))]
    output_folder: String,

    /// Include dev dependencies [default: excluded]
    #[arg(short, long)]
    dev: bool,

    /// Include build dependencies [default: excluded]
    #[arg(short, long)]
    build: bool,

    /// Exclude specified workspace [default: all included]
    #[arg(short, long)]
    exclude: Vec<String>,

    /// The depth of dependencies to collect licenses for [default: all sub dependencies]
    #[arg(short = 'D', long)]
    depth: Option<u8>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let _ = std::fs::remove_dir_all(&args.output_folder);
    std::fs::create_dir(&args.output_folder).context("failed to create output folder")?;

    let crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;

    let all_packages = try_get_packages()?;

    copy_licenses(
        FileSystem {},
        crates,
        all_packages,
        PathBuf::from(args.output_folder),
    )?;

    Ok(())
}

fn copy_licenses(
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
