use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Output};

fn call_licenses_command(args: &[&str]) -> Output {
    if !std::fs::exists("target/release/cargo-licenses").unwrap() {
        panic!("cargo-licenses has not been built")
    }
    Command::new("target/release/cargo-licenses")
        .arg("licenses")
        .args(args)
        .output()
        .unwrap()
}

fn collected_dependencies(dir: &Path) -> HashSet<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.split('-').next().map(ToOwned::to_owned))
        .collect()
}

fn actual_dependencies() -> HashSet<String> {
    cargo_toml::Manifest::from_path("./Cargo.toml")
        .unwrap()
        .dependencies
        .into_keys()
        .collect()
}

#[test]
fn help_flag() {
    let output = call_licenses_command(&["--help"]);
    assert!(output.status.success());
    assert_eq!(
        include_str!("stdout/help"),
        String::from_utf8(output.stdout).unwrap()
    )
}

#[test]
fn summary_depth_1() {
    let output = call_licenses_command(&["summary", "--depth", "1"]);
    assert!(output.status.success());
    assert_eq!(
        include_str!("stdout/summary"),
        String::from_utf8(output.stdout).unwrap()
    )
}

#[test]
fn collect_depth_1() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path();

    let output = call_licenses_command(&[
        "collect",
        "--depth",
        "1",
        "--path",
        temp_dir_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert_eq!(collected_dependencies(temp_dir_path), actual_dependencies());
}
