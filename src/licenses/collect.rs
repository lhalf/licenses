use crate::cargo_metadata::Package;
use crate::config::CrateConfig;
use crate::file_io::DirEntry;
use crate::file_io::FileIO;
use crate::licenses::is_license::is_license;
use std::collections::HashMap;

pub fn collect_licenses(
    file_io: &impl FileIO,
    packages: &[Package],
    crate_configs: &HashMap<String, CrateConfig>,
) -> anyhow::Result<HashMap<Package, Vec<DirEntry>>> {
    packages
        .iter()
        .map(|package| {
            collect_licenses_for_package(
                file_io,
                package,
                skipped_files_for_package(package, crate_configs),
            )
        })
        .collect()
}

fn skipped_files_for_package<'a>(
    package: &'a Package,
    crate_configs: &'a HashMap<String, CrateConfig>,
) -> &'a [String] {
    crate_configs
        .get(&package.normalised_name)
        .map(|config| config.skip.as_slice())
        .unwrap_or(&[])
}

fn collect_licenses_for_package(
    file_io: &impl FileIO,
    package: &Package,
    skipped_files: &[String],
) -> anyhow::Result<(Package, Vec<DirEntry>)> {
    Ok((
        package.clone(),
        file_io
            .read_dir(package.path.as_ref())?
            .into_iter()
            .filter(is_license)
            .filter(|dir_entry| !is_skipped_file(dir_entry, skipped_files))
            .collect(),
    ))
}

fn is_skipped_file(dir_entry: &DirEntry, skipped_files: &[String]) -> bool {
    dir_entry
        .name
        .to_str()
        .map(|file_name| skipped_files.contains(&file_name.to_string()))
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::config::CrateConfig;
    use crate::file_io::{DirEntry, FileIOSpy};
    use crate::licenses::collect::collect_licenses;
    use std::collections::HashMap;
    use std::ffi::OsString;

    #[test]
    fn failure_to_read_dir_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .set([Err(anyhow::anyhow!("deliberate test error"))]);

        assert!(
            collect_licenses(&file_io_spy, &[Package::called("example")], &HashMap::new()).is_err()
        );
    }

    #[test]
    fn no_files_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(Vec::new())]);

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), Vec::new())]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &HashMap::new()).unwrap()
        );
    }

    #[test]
    fn no_licenses_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("some_file"),
            path: Default::default(),
            is_file: true,
        }])]);

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), Vec::new())]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &HashMap::new()).unwrap()
        );
    }

    #[test]
    fn single_license_in_directory() {
        let file_io_spy = FileIOSpy::default();
        let dir_entry = DirEntry {
            name: OsString::from("LICENSE"),
            path: Default::default(),
            is_file: true,
        };
        file_io_spy
            .read_dir
            .returns
            .set([Ok(vec![dir_entry.clone()])]);

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), vec![dir_entry])]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &HashMap::new()).unwrap()
        );
    }

    #[test]
    fn single_skipped_license_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![DirEntry {
            name: OsString::from("LICENSE"),
            path: Default::default(),
            is_file: true,
        }])]);

        let skipped_files: HashMap<_, _> = [(
            "example".to_string(),
            CrateConfig {
                skip: vec!["LICENSE".to_string()],
                allow: None,
            },
        )]
        .into_iter()
        .collect();

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), Vec::new())]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &skipped_files).unwrap()
        );
    }

    #[test]
    fn multiple_skipped_licenses_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.set([Ok(vec![
            DirEntry {
                name: OsString::from("LICENSE-MIT"),
                path: Default::default(),
                is_file: true,
            },
            DirEntry {
                name: OsString::from("LICENSE-APACHE"),
                path: Default::default(),
                is_file: true,
            },
            DirEntry {
                name: OsString::from("COPYRIGHT"),
                path: Default::default(),
                is_file: true,
            },
        ])]);

        let skipped_files: HashMap<_, _> = [(
            "example".to_string(),
            CrateConfig {
                skip: vec!["COPYRIGHT".to_string(), "LICENSE-APACHE".to_string()],
                allow: None,
            },
        )]
        .into_iter()
        .collect();

        let expected_licenses: HashMap<_, _> = [(
            Package::called("example"),
            vec![DirEntry {
                name: OsString::from("LICENSE-MIT"),
                path: Default::default(),
                is_file: true,
            }],
        )]
        .into_iter()
        .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &skipped_files).unwrap()
        );
    }
}
