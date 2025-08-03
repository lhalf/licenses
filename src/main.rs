use crate::cargo_metadata::{Package, try_get_packages};
use crate::cargo_tree::crate_names;
use crate::copy_licenses::copy_licenses;
use crate::file_io::FileSystem;
use crate::summarise::{crates_per_license, summarise};
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod cargo_metadata;
mod cargo_tree;
mod copy_licenses;
mod file_io;
mod is_license;
mod license;
mod macros;
mod split_licenses;
mod summarise;
mod validate_licenses;

#[derive(Parser)]
#[command(bin_name = "cargo", disable_help_subcommand = true)]
enum CargoSubcommand {
    #[command(name = "licenses", version, author, disable_version_flag = true)]
    Licenses {
        #[command(flatten)]
        args: GlobalArgs,

        #[command(subcommand)]
        command: LicensesSubcommand,
    },
}

#[derive(Args)]
struct GlobalArgs {
    /// Include dev dependencies [default: excluded]
    #[arg(short, long, global = true)]
    dev: bool,

    /// Include build dependencies [default: excluded]
    #[arg(short, long, global = true)]
    build: bool,

    /// Exclude specified workspace [default: all included]
    #[arg(short, long, value_name = "WORKSPACE", global = true)]
    exclude: Vec<String>,

    /// The depth of dependencies to include [default: all sub dependencies]
    #[arg(short = 'D', long, global = true)]
    depth: Option<u8>,
}

#[derive(Subcommand)]
enum LicensesSubcommand {
    /// Collects all licenses into a folder
    Collect {
        /// The output license folder path
        #[arg(short, long, default_value_t = String::from("licenses"))]
        path: String,
    },
    /// Provides a summary of all licenses
    Summary {
        /// Display the summary as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let CargoSubcommand::Licenses { args, command } = CargoSubcommand::parse();

    let crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;
    let all_packages = try_get_packages()?;
    let filtered_packages = filter_packages(crates, all_packages);

    match command {
        LicensesSubcommand::Collect { path } => {
            let path = PathBuf::from(path);
            create_output_folder(&path)?;
            copy_licenses(FileSystem {}, filtered_packages, path)?;
        }
        LicensesSubcommand::Summary { json } => {
            let crates_per_license = crates_per_license(filtered_packages);
            println!(
                "{}",
                match json {
                    true => serde_json::to_string_pretty(&crates_per_license)?,
                    false => summarise(crates_per_license),
                }
            )
        }
    }

    Ok(())
}

fn create_output_folder(path: &Path) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir(path).context("failed to create output folder")
}

fn filter_packages(crates: BTreeSet<String>, all_packages: Vec<Package>) -> Vec<Package> {
    all_packages
        .into_iter()
        .filter(|package| crates.contains(&package.normalised_name))
        .collect()
}
