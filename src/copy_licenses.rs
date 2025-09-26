use crate::cargo_metadata::Package;
use crate::collect_licenses::collect_licenses;
use crate::file_io::FileIO;
use crate::license::License;
use crate::validate_licenses::validate_licenses;
use anyhow::Context;
use std::path::PathBuf;

pub fn copy_licenses(
    file_io: impl FileIO,
    filtered_packages: Vec<Package>,
    output_folder: PathBuf,
) -> anyhow::Result<()> {
    for package in filtered_packages {
        let licenses = collect_licenses(&file_io, &package, Vec::new())?;

        validate_licenses(
            &file_io,
            &package.license.as_deref().map(License::parse),
            &licenses,
        )
        .warn(&package);

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

    println!("{}", output_folder.to_string_lossy());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::copy_licenses::copy_licenses;
    use crate::file_io::{DirEntry, FileIOSpy};
    use cargo_metadata::camino::Utf8PathBuf;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn no_filtered_packages_causes_no_directory_read_or_files_copied() {
        let file_io_spy = FileIOSpy::default();
        assert!(copy_licenses(file_io_spy.clone(), Vec::new(), PathBuf::default()).is_ok());
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
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
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
            is_file: false,
        }]));
        assert!(
            copy_licenses(
                file_io_spy.clone(),
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Utf8PathBuf::from("/example"),
                    url: None,
                    license: None,
                }],
                PathBuf::default()
            )
            .is_ok()
        );
        assert_eq!(
            vec![PathBuf::from("/example")],
            file_io_spy.read_dir.arguments.take_all()
        );
        assert!(file_io_spy.copy_file.arguments.take_all().is_empty());
    }
}
