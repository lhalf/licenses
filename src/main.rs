use crate::cargo_metadata::try_get_packages;
use crate::cargo_tree::crate_names;
use anyhow::Context;
use clap::Parser;

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

    let _crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;

    let _all_packages = try_get_packages()?;

    Ok(())
}
