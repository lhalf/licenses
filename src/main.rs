use clap::Parser;
use crate::cargo_tree::crate_names;

mod cargo_tree;
mod file_system;
mod find_licenses;
mod find_and_copy_licenses;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The output license folder path
    #[arg(short, long, default_value_t = String::from("licenses"))]
    output_file: String,

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
    let _crates = crate_names(args.depth, args.dev, args.build, args.exclude)?;
    Ok(())
}
