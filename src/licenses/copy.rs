use crate::cargo_metadata::Package;
use crate::config::{CrateConfig, IncludedLicense};
use crate::file_io::{DirEntry, FileIO};
use anyhow::Context;
use std::collections::HashMap;
use std::path::Path;

pub fn copy_licenses(
    file_io: &impl FileIO,
    all_licenses: HashMap<Package, Vec<DirEntry>>,
    output_folder: &Path,
    crate_configs: &HashMap<String, CrateConfig>,
) -> anyhow::Result<()> {
    for (package, licenses) in all_licenses {
        copy_licenses_to_output_folder(file_io, &licenses, output_folder, &package)?;
        add_included_licenses_to_output_folder(file_io, output_folder, &package, crate_configs)?;
    }

    println!("{}", output_folder.to_string_lossy());
    Ok(())
}

fn copy_licenses_to_output_folder(
    file_io: &impl FileIO,
    licenses: &[DirEntry],
    output_folder: &Path,
    package: &Package,
) -> anyhow::Result<()> {
    for license in licenses {
        file_io.copy_file(
            &license.path,
            &output_folder.join(
                package.license_filename(
                    &license
                        .path
                        .file_name()
                        .context("license name contained invalid UTF-8")?
                        .to_string_lossy(),
                ),
            ),
        )?;
    }
    Ok(())
}

fn add_included_licenses_to_output_folder(
    file_io: &impl FileIO,
    output_folder: &Path,
    package: &Package,
    crate_configs: &HashMap<String, CrateConfig>,
) -> anyhow::Result<()> {
    if let Some(config) = crate_configs.get(&package.normalised_name) {
        for included_license in &config.include {
            match included_license {
                IncludedLicense::Text { name, text } => {
                    file_io.write_file(&output_folder.join(package.license_filename(name)), text)?
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::licenses::copy::copy_licenses;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn no_licenses_causes_no_files_copied() {
        let file_io_spy = FileIOSpy::default();

        assert!(
            copy_licenses(
                &file_io_spy,
                HashMap::new(),
                &PathBuf::new(),
                &HashMap::new()
            )
            .is_ok()
        );

        assert_eq!(file_io_spy.copy_file.arguments.take(), []);
    }

    #[test]
    fn failure_to_copy_file_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .copy_file
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        let all_licenses = vec![(
            Package::called("example"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example/LICENSE"),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        assert_eq!(
            "deliberate test error",
            copy_licenses(&file_io_spy, all_licenses, &PathBuf::new(), &HashMap::new())
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn included_licenses_are_written_for_each_version() {
        use crate::config::{CrateConfig, IncludedLicense};

        let file_io_spy = FileIOSpy::default();
        file_io_spy.write_file.returns.set([Ok(()), Ok(())]);

        let mut first_version = Package::called("example");
        first_version.version = "1.0.0".to_string();
        let mut second_version = first_version.clone();
        second_version.version = "2.0.0".to_string();
        let mut all_licenses = HashMap::new();
        all_licenses.insert(first_version, vec![]);
        all_licenses.insert(second_version, vec![]);

        let crate_configs = HashMap::from([(
            "example".to_string(),
            CrateConfig {
                include: vec![IncludedLicense::Text {
                    name: "LICENSE-CUSTOM".to_string(),
                    text: "custom license text".to_string(),
                }],
                ..CrateConfig::default()
            },
        )]);

        assert!(
            copy_licenses(
                &file_io_spy,
                all_licenses,
                &PathBuf::from("output"),
                &crate_configs
            )
            .is_ok()
        );

        let mut write_args = file_io_spy.write_file.arguments.take();
        write_args.sort();
        assert_eq!(write_args.len(), 2);
        assert_eq!(
            write_args[0].0,
            PathBuf::from("output/example-1.0.0-LICENSE-CUSTOM")
        );
        assert_eq!(write_args[0].1, "custom license text");
        assert_eq!(
            write_args[1].0,
            PathBuf::from("output/example-2.0.0-LICENSE-CUSTOM")
        );
        assert_eq!(write_args[1].1, "custom license text");
    }

    #[test]
    fn multiple_licenses_for_single_package_are_all_copied() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.copy_file.returns.set([Ok(()), Ok(())]);

        let all_licenses = vec![(
            Package::called("example"),
            vec![
                DirEntry {
                    name: OsString::from("LICENSE-MIT"),
                    path: PathBuf::from("example/LICENSE-MIT"),
                    is_file: true,
                },
                DirEntry {
                    name: OsString::from("LICENSE-APACHE"),
                    path: PathBuf::from("example/LICENSE-APACHE"),
                    is_file: true,
                },
            ],
        )]
        .into_iter()
        .collect();

        assert!(
            copy_licenses(
                &file_io_spy,
                all_licenses,
                &PathBuf::from("output"),
                &HashMap::new()
            )
            .is_ok()
        );

        let copy_args = file_io_spy.copy_file.arguments.take();
        assert_eq!(2, copy_args.len());
    }

    #[test]
    fn copy_writes_to_correct_output_path() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.copy_file.returns.set([Ok(())]);

        let all_licenses = vec![(
            Package::called("my_crate"),
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("/src/my_crate/LICENSE"),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        assert!(
            copy_licenses(
                &file_io_spy,
                all_licenses,
                &PathBuf::from("licenses"),
                &HashMap::new()
            )
            .is_ok()
        );

        let copy_args = file_io_spy.copy_file.arguments.take();
        assert_eq!(
            copy_args[0],
            (
                PathBuf::from("/src/my_crate/LICENSE"),
                PathBuf::from("licenses/my_crate-0.0.0-LICENSE")
            )
        );
    }

    #[test]
    fn different_versions_copy_licenses_to_distinct_paths() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.copy_file.returns.set([Ok(()), Ok(())]);

        let mut first_version = Package::called("example");
        first_version.version = "1.0.0".to_string();
        first_version.license = Some("MIT".to_string());
        let mut second_version = first_version.clone();
        second_version.version = "2.0.0".to_string();
        let mut all_licenses = HashMap::new();
        all_licenses.insert(
            first_version,
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example-1.0.0/LICENSE"),
                is_file: true,
            }],
        );
        all_licenses.insert(
            second_version,
            vec![DirEntry {
                name: OsString::from("LICENSE"),
                path: PathBuf::from("example-2.0.0/LICENSE"),
                is_file: true,
            }],
        );

        copy_licenses(
            &file_io_spy,
            all_licenses,
            &PathBuf::from("licenses"),
            &HashMap::new(),
        )
        .unwrap();

        let mut copy_args = file_io_spy.copy_file.arguments.take();
        copy_args.sort();
        assert_eq!(copy_args.len(), 2);
        assert_eq!(copy_args[0].0, PathBuf::from("example-1.0.0/LICENSE"));
        assert_eq!(
            copy_args[0].1,
            PathBuf::from("licenses/example-1.0.0-LICENSE")
        );
        assert_eq!(copy_args[1].0, PathBuf::from("example-2.0.0/LICENSE"));
        assert_eq!(
            copy_args[1].1,
            PathBuf::from("licenses/example-2.0.0-LICENSE")
        );
    }
}
