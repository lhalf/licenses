use crate::cargo_metadata::Package;
use crate::config::{CrateConfig, IncludedLicense};
use crate::file_io::{DirEntry, FileIO};
use anyhow::Context;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn copy_licenses(
    file_io: &impl FileIO,
    all_licenses: HashMap<Package, Vec<DirEntry>>,
    output_folder: PathBuf,
    crate_configs: &HashMap<String, CrateConfig>,
) -> anyhow::Result<()> {
    for (package, licenses) in all_licenses {
        copy_licenses_to_output_folder(file_io, &licenses, &output_folder, &package)?;
        add_included_licenses_to_output_folder(file_io, &output_folder, &package, crate_configs)?;
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
                IncludedLicense::Text { name, text } => file_io.write_file(
                    &output_folder.join(format!("{}-{name}", package.normalised_name)),
                    text,
                )?,
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
    fn no_licenses_causes_no_files_copied_and_no_logs() {
        let file_io_spy = FileIOSpy::default();

        assert!(
            copy_licenses(
                &file_io_spy,
                HashMap::new(),
                PathBuf::default(),
                &HashMap::new()
            )
            .is_ok()
        );

        assert!(file_io_spy.copy_file.arguments.take().is_empty());
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
            copy_licenses(
                &file_io_spy,
                all_licenses,
                PathBuf::default(),
                &HashMap::new()
            )
            .unwrap_err()
            .to_string()
        );
    }

    // TODO
    // add test for included file calling write_file
}
