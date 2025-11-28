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

#[test]
fn help_flag() {
    let output = call_licenses_command(&["--help"]);
    assert!(output.status.success());
    assert_eq!(
        include_str!("stdout/help"),
        String::from_utf8(output.stdout).unwrap()
    )
}
