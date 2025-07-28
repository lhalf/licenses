use crate::cargo_metadata::try_get_packages;
use crate::cargo_tree::crate_names;
use crate::copy_licenses::copy_licenses;
use crate::file_io::FileSystem;
use crate::summarise::summarise;
use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod cargo_metadata;
mod cargo_tree;
mod copy_licenses;
mod file_io;
mod is_license;
mod macros;
mod split_licenses;
mod summarise;
mod validate_licenses;

#[derive(Parser)]
#[command(bin_name = "cargo", disable_help_subcommand = true)]
enum CargoSubcommand {
    #[command(name = "licenses", version, author, disable_version_flag = true)]
    Licenses(Licenses),
}

#[derive(Parser)]
struct Licenses {
    /// Include dev dependencies [default: excluded]
    #[arg(short, long)]
    dev: bool,

    /// Include build dependencies [default: excluded]
    #[arg(short, long)]
    build: bool,

    /// Exclude specified workspace [default: all included]
    #[arg(short, long, value_name = "WORKSPACE")]
    exclude: Vec<String>,

    /// The depth of dependencies to collect licenses for [default: all sub dependencies]
    #[arg(short = 'D', long)]
    depth: Option<u8>,

    #[command(subcommand)]
    command: LicensesSubcommand,
}

#[derive(Subcommand)]
enum LicensesSubcommand {
    /// Collects all licenses into a folder
    Folder {
        /// The output license folder path
        #[arg(short, long, default_value_t = String::from("licenses"))]
        path: String,
    },
    /// Provides a summary of all licenses
    Summary,
}

fn main() -> anyhow::Result<()> {
    let CargoSubcommand::Licenses(args) = CargoSubcommand::parse();

    let crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;

    let all_packages = try_get_packages()?;

    match args.command {
        LicensesSubcommand::Folder { path } => {
            let path = PathBuf::from(path);
            create_output_folder(&path)?;
            copy_licenses(FileSystem {}, crates, all_packages, path)?;
        }
        LicensesSubcommand::Summary => {
            summarise(crates, all_packages);
        }
    }

    Ok(())
}

fn create_output_folder(path: &Path) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir(path).context("failed to create output folder")
}
