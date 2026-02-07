use crate::cargo_metadata::{filtered_packages, try_get_packages};
use crate::cargo_tree::crate_names;
use crate::config::load_config;
use crate::file_io::FileSystem;
use crate::licenses::subcommand;
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use std::path::PathBuf;

mod cargo_metadata;
mod cargo_tree;
mod config;
mod file_io;
mod licenses;
mod log;

fn main() -> anyhow::Result<()> {
    let CargoSubcommand::Licenses { args, command } = CargoSubcommand::parse();

    let file_system = FileSystem {};
    let config = load_config(&file_system, args)?;
    let filtered_packages = filtered_packages(try_get_packages()?, &crate_names(&config)?);

    match command {
        LicensesSubcommand::Collect { path } => {
            subcommand::collect(&file_system, &config, &filtered_packages, path)?;
        }
        LicensesSubcommand::Summary(args) => {
            subcommand::summary(filtered_packages, &args)?;
        }
        LicensesSubcommand::Check => {
            subcommand::check(&file_system, &config, &filtered_packages)?;
        }
        LicensesSubcommand::Diff { path } => {
            subcommand::diff(&file_system, &config, &filtered_packages, path)?;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(bin_name = "cargo", disable_help_subcommand = true)]
enum CargoSubcommand {
    #[command(
        name = "licenses",
        version,
        author,
        disable_version_flag = true,
        override_usage = "cargo licenses <COMMAND> [OPTIONS]"
    )]
    Licenses {
        #[command(flatten)]
        args: GlobalArgs,

        #[command(subcommand)]
        command: LicensesSubcommand,
    },
}

#[derive(Debug, Args, Deserialize, PartialEq, Eq, Default, Clone)]
#[serde(default, deny_unknown_fields)]
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

    /// Activate all features [default: default features]
    #[arg(long, global = true)]
    #[serde(rename = "all-features")]
    all_features: bool,

    /// Do not activate default features [default: default features]
    #[arg(long, global = true)]
    #[serde(rename = "no-default-features")]
    no_default_features: bool,

    /// Exclude specified workspace [default: all included]
    #[arg(short, long, value_name = "WORKSPACE", global = true)]
    exclude: Vec<String>,

    /// Ignore specified crate [default: all included]
    #[arg(short, long, value_name = "CRATE", global = true)]
    ignore: Vec<String>,

    /// Path to configuration file
    #[arg(short, long, value_name = "PATH", global = true)]
    #[serde(skip)]
    config: Option<PathBuf>,
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
    /// Diff between the current licenses folder and the licenses that would be collected
    Diff {
        /// The current licenses folder path
        #[arg(short, long, default_value_t = String::from("licenses"))]
        path: String,
    },
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
