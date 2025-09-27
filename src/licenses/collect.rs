use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::file_io::FileIO;
use crate::licenses::is_license::is_license;
use std::collections::HashMap;

pub fn collect_licenses(
    file_io: &impl FileIO,
    packages: &[Package],
    skipped_files: &[String],
) -> anyhow::Result<HashMap<Package, Vec<DirEntry>>> {
    packages
        .iter()
        .map(|package| collect_licenses_for_package(file_io, package, skipped_files))
        .collect()
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
            .filter(|dir_entry| !is_skipped_file(dir_entry, skipped_files, package))
            .collect(),
    ))
}

fn is_skipped_file(dir_entry: &DirEntry, skipped_files: &[String], package: &Package) -> bool {
    dir_entry
        .name
        .to_str()
        .map(|file_name| {
            skipped_files.contains(&format!("{}-{}", package.normalised_name, file_name))
        })
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
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
            .push_back(Err(anyhow::anyhow!("deliberate test error")));

        assert!(collect_licenses(&file_io_spy, &[Package::called("example")], &[]).is_err());
    }

    #[test]
    fn no_files_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.push_back(Ok(Vec::new()));

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), Vec::new())]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &[]).unwrap()
        );
    }

    #[test]
    fn no_licenses_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.push_back(Ok(vec![DirEntry {
            name: OsString::from("some_file"),
            path: Default::default(),
            is_file: true,
        }]));

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), Vec::new())]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &[]).unwrap()
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
            .push_back(Ok(vec![dir_entry.clone()]));

        let expected_licenses: HashMap<_, _> = [(Package::called("example"), vec![dir_entry])]
            .into_iter()
            .collect();

        assert_eq!(
            expected_licenses,
            collect_licenses(&file_io_spy, &[Package::called("example")], &[]).unwrap()
        );
    }

    #[test]
    fn single_skipped_license_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.push_back(Ok(vec![DirEntry {
            name: OsString::from("LICENSE"),
            path: Default::default(),
            is_file: true,
        }]));

        let skipped_files = ["example-LICENSE".to_string()];

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
        file_io_spy.read_dir.returns.push_back(Ok(vec![
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
        ]));

        let skipped_files = [
            "example-COPYRIGHT".to_string(),
            "example-LICENSE-APACHE".to_string(),
        ];

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
