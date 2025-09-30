use crate::cargo_metadata::{Package, try_get_packages};
use crate::cargo_tree::crate_names;
use crate::config::load_config;
use crate::file_io::FileSystem;
use crate::licenses::check::check_licenses;
use crate::licenses::collect::collect_licenses;
use crate::licenses::copy::copy_licenses;
use crate::licenses::summarise::{crates_per_license, summarise};
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod cargo_metadata;
mod cargo_tree;
mod config;
mod file_io;
mod licenses;
mod macros;

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

#[derive(Debug, Args, Deserialize, PartialEq, Default, Clone)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct GlobalArgs {
    /// Include dev dependencies [default: excluded]
    #[arg(short, long, global = true)]
    dev: bool,

    /// Include build dependencies [default: excluded]
    #[arg(short, long, global = true)]
    build: bool,

    /// The depth of dependencies to include [default: all sub dependencies]
    #[arg(short = 'D', long, global = true)]
    depth: Option<u8>,

    /// Exclude specified workspace [default: all included]
    #[arg(short, long, value_name = "WORKSPACE", global = true)]
    exclude: Vec<String>,

    /// Ignore specified crate [default: all included]
    #[arg(short, long, value_name = "CRATE", global = true)]
    ignore: Vec<String>,

    /// Path to configuration file
    #[arg(short, long, value_name = "PATH", global = true)]
    #[serde(skip)]
    config: Option<String>,
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
    Summary(SummaryArgs),
    /// Checks all licenses for inconsistencies
    Check,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
struct SummaryArgs {
    /// Display the summary as JSON
    #[arg(long)]
    json: bool,
    /// Display the summary as TOML
    #[arg(long)]
    toml: bool,
}

fn main() -> anyhow::Result<()> {
    let CargoSubcommand::Licenses { args, command } = CargoSubcommand::parse();

    let file_system = FileSystem {};
    let config = load_config(&file_system, args)?;
    let crates_we_want = crate_names(
        config.global.depth,
        config.global.dev,
        config.global.build,
        config.global.exclude,
        config.global.ignore,
    )?;
    let filtered_packages = filter_packages(try_get_packages()?, crates_we_want);

    match command {
        LicensesSubcommand::Collect { path } => {
            let path = PathBuf::from(path);
            create_output_folder(&path)?;
            copy_licenses(
                &file_system,
                collect_licenses(&file_system, &filtered_packages, &config.crates)?,
                path,
            )?;
        }
        LicensesSubcommand::Summary(args) => {
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
            )
        }
        LicensesSubcommand::Check => {
            if check_licenses(
                &file_system,
                collect_licenses(&file_system, &filtered_packages, &config.crates)?,
            )
            .is_err()
            {
                std::process::exit(1)
            }
        }
    }

    Ok(())
}

fn create_output_folder(path: &Path) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir(path).context("failed to create output folder")
}

fn filter_packages(all_packages: Vec<Package>, crates_we_want: BTreeSet<String>) -> Vec<Package> {
    all_packages
        .into_iter()
        .filter(|package| crates_we_want.contains(&package.normalised_name))
        .collect()
}
