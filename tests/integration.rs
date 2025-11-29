fn call_licenses_command(args: &[&str]) -> std::process::Output {
    if !std::fs::exists("target/release/cargo-licenses").unwrap() {
        panic!("cargo-licenses has not been built")
    }
    std::process::Command::new("target/release/cargo-licenses")
        .arg("licenses")
        .args(args)
        .output()
        .unwrap()
}

fn list_files(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
        })
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

    let expected_licenses = vec![
        "once_cell-LICENSE-APACHE",
        "anyhow-LICENSE-APACHE",
        "serde-LICENSE-APACHE",
        "spdx-LICENSE-MIT",
        "colored-LICENSE",
        "itertools-LICENSE-MIT",
        "once_cell-LICENSE-MIT",
        "toml-LICENSE-APACHE",
        "spdx-LICENSE-APACHE",
        "itertools-LICENSE-APACHE",
        "indicatif-LICENSE",
        "serde_json-LICENSE-APACHE",
        "anyhow-LICENSE-MIT",
        "serde-LICENSE-MIT",
        "cargo_metadata-LICENSE-MIT",
        "clap-LICENSE-APACHE",
        "serde_json-LICENSE-MIT",
        "strsim-LICENSE",
        "clap-LICENSE-MIT",
        "toml-LICENSE-MIT",
    ];

    assert!(output.status.success());
    assert_eq!(expected_licenses, list_files(temp_dir_path))
}
