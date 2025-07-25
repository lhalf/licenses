use crate::cargo_metadata::{Package, try_get_packages};
use crate::cargo_tree::crate_names;
use crate::copy_licenses::copy_licenses;
use crate::file_io::FileSystem;
use anyhow::Context;
use clap::{Parser, Subcommand};
use std::collections::BTreeSet;
use std::path::PathBuf;

mod cargo_metadata;
mod cargo_tree;
mod copy_licenses;
mod file_io;
mod is_license;
mod macros;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Collects all licenses into a folder
    Folder {
        /// The output license folder path
        #[arg(short, long, default_value_t = String::from("licenses"))]
        path: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;

    let all_packages = try_get_packages()?;

    match args.command {
        Commands::Folder { path } => {
            copy_licenses_to_folder(PathBuf::from(path), crates, all_packages)
        }
    }
}

fn copy_licenses_to_folder(
    folder: PathBuf,
    crates: BTreeSet<String>,
    all_packages: Vec<Package>,
) -> Result<(), anyhow::Error> {
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).context("failed to create output folder")?;

    copy_licenses(FileSystem {}, crates, all_packages, folder)?;

    Ok(())
}
