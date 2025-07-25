use crate::cargo_metadata::Package;
use crate::file_io::{DirEntry, FileIO};
use anyhow::Context;
use colored::Colorize;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn copy_licenses(
    file_io: impl FileIO,
    crates: BTreeSet<String>,
    all_packages: Vec<Package>,
    output_folder: PathBuf,
) -> anyhow::Result<()> {
    for package in all_packages {
        if !crates.contains(&package.normalised_name) {
            continue;
        }

        let licenses: Vec<DirEntry> = file_io
            .read_dir(package.path.as_ref())?
            .into_iter()
            .filter(|dir_entry| {
                file_io.is_file(&dir_entry.path)
                    && dir_entry
                        .name
                        .to_string_lossy()
                        .to_lowercase()
                        .starts_with("license")
            })
            .collect();

        if licenses.is_empty() {
            println!(
                "{}: did not find any licenses for {} - try looking here: {}",
                "warning".yellow().bold(),
                package.normalised_name,
                package.url
            );
            continue;
        }

        for license in licenses {
            file_io.copy_file(
                &license.path,
                &output_folder.join(format!(
                    "{}-{}",
                    package.normalised_name,
                    license
                        .path
                        .file_name()
                        .context("license name contained invalid UTF-8")?
                        .to_string_lossy()
                )),
            )?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::copy_licenses::copy_licenses;
    use crate::file_io::{DirEntry, FileIOSpy};
    use cargo_metadata::camino::Utf8PathBuf;
    use std::collections::BTreeSet;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn no_packages_causes_no_directory_read_or_files_copied() {
        let file_io_spy = FileIOSpy::default();
        assert!(
            copy_licenses(
                file_io_spy.clone(),
                BTreeSet::new(),
                Vec::new(),
                PathBuf::default()
            )
            .is_ok()
        );
        assert!(file_io_spy.read_dir.arguments.take_all().is_empty());
        assert!(file_io_spy.copy_file.arguments.take_all().is_empty());
    }

    #[test]
    fn no_crates_required_causes_no_directory_read_or_files_copied() {
        let file_io_spy = FileIOSpy::default();
        assert!(
            copy_licenses(
                file_io_spy.clone(),
                BTreeSet::new(),
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Utf8PathBuf::from("/example"),
                    url: String::new(),
                }],
                PathBuf::default()
            )
            .is_ok()
        );
        assert!(file_io_spy.read_dir.arguments.take_all().is_empty());
        assert!(file_io_spy.copy_file.arguments.take_all().is_empty());
    }

    #[test]
    fn failure_to_read_dir_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .push_back(Err(anyhow::anyhow!("deliberate test error")));
        assert_eq!(
            "deliberate test error",
            copy_licenses(
                file_io_spy.clone(),
                BTreeSet::from(["example".to_string()]),
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: String::new(),
                }],
                PathBuf::default()
            )
            .unwrap_err()
            .to_string()
        );
    }

    #[test]
    fn does_not_copy_directories_starting_with_license() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.push_back(Ok(vec![DirEntry {
            name: OsString::from("license_directory"),
            path: Default::default(),
        }]));
        file_io_spy.is_file.returns.push_back(false);
        assert!(
            copy_licenses(
                file_io_spy.clone(),
                BTreeSet::from(["example".to_string()]),
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Utf8PathBuf::from("/example"),
                    url: String::new(),
                }],
                PathBuf::default()
            )
                .is_ok()
        );
        assert_eq!(vec![PathBuf::from("/example")], file_io_spy.read_dir.arguments.take_all());
        assert!(file_io_spy.copy_file.arguments.take_all().is_empty());
    }
}
