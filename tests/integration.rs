use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Output};

fn call_licenses_command(args: &[&str]) -> Output {
    assert!(
        std::fs::exists("target/release/cargo-licenses").unwrap(),
        "cargo-licenses has not been built"
    );
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
    );
}

#[test]
fn summary_depth_1() {
    let output = call_licenses_command(&["summary", "--depth", "1"]);
    assert!(output.status.success());
    assert_eq!(
        include_str!("stdout/summary"),
        String::from_utf8(output.stdout).unwrap()
    );
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

#[test]
fn check_warns_about_unused_config() {
    let output = call_licenses_command(&["check", "--config", "tests/data/unused_config.toml"]);

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = strip_ansi_escapes::strip_str(&stdout);

    assert!(stdout.contains("anyhow - 'allow' is not required"));
    assert!(stdout.contains("fake_crate - crate not found in dependencies"));
    assert!(stdout.contains("strsim - 'skip' for NONEXISTENT is not required"));
}

#[test]
fn check_depth_1_succeeds() {
    let output = call_licenses_command(&["check", "--depth", "1"]);
    assert!(output.status.success());
}

#[test]
fn diff_depth_1_succeeds_when_licenses_folder_matches() {
    let output = call_licenses_command(&["diff", "--depth", "1", "--path", "licenses"]);
    assert!(output.status.success());
}

#[test]
fn diff_reports_differences_for_empty_folder() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let output = call_licenses_command(&[
        "diff",
        "--depth",
        "1",
        "--path",
        temp_dir.path().to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = strip_ansi_escapes::strip_str(&stdout);
    assert!(stdout.contains("missing"));
}

#[test]
fn summary_json_depth_1() {
    let output = call_licenses_command(&["summary", "--depth", "1", "--json"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn summary_toml_depth_1() {
    let output = call_licenses_command(&["summary", "--depth", "1", "--toml"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.parse::<toml::Table>().is_ok());
}

#[test]
fn collect_into_temp_dir_creates_expected_files() {
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

    // every file should be prefixed with a crate name
    let files: Vec<String> = std::fs::read_dir(temp_dir_path)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    assert!(!files.is_empty());
    // all files should contain a dash (crate_name-LICENSE_FILE format)
    assert!(files.iter().all(|f| f.contains('-')));
}

#[test]
fn invalid_subcommand_fails() {
    let output = Command::new("target/release/cargo-licenses")
        .arg("licenses")
        .arg("nonexistent")
        .output()
        .unwrap();

    assert!(!output.status.success());
}
