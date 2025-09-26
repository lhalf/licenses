use crate::cargo_metadata::Package;
use crate::file_io::DirEntry;
use crate::file_io::FileIO;
use crate::is_license::is_license;

pub fn collect_licenses(
    file_io: &impl FileIO,
    package: &Package,
    skipped_files: &[String],
) -> anyhow::Result<Vec<DirEntry>> {
    let licenses: Vec<DirEntry> = file_io
        .read_dir(package.path.as_ref())?
        .into_iter()
        .filter(is_license)
        .filter(|dir_entry| {
            dir_entry
                .name
                .to_str()
                .map(|file_name| {
                    !skipped_files.contains(&format!("{}-{}", package.normalised_name, file_name))
                })
                .unwrap_or(false)
        })
        .collect();
    Ok(licenses)
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::collect_licenses::collect_licenses;
    use crate::file_io::{DirEntry, FileIOSpy};
    use std::ffi::OsString;

    #[test]
    fn failure_to_read_dir_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .push_back(Err(anyhow::anyhow!("deliberate test error")));
        assert!(
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn nothing_in_directory() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy.read_dir.returns.push_back(Ok(Vec::new()));
        assert_eq!(
            Vec::<DirEntry>::new(),
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &[]
            )
            .unwrap()
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
        assert_eq!(
            Vec::<DirEntry>::new(),
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &[]
            )
            .unwrap()
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
        assert_eq!(
            vec![dir_entry],
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &[]
            )
            .unwrap()
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
        assert_eq!(
            Vec::<DirEntry>::new(),
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &["example-LICENSE".to_string()]
            )
            .unwrap()
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
        assert_eq!(
            vec![DirEntry {
                name: OsString::from("LICENSE-MIT"),
                path: Default::default(),
                is_file: true,
            },],
            collect_licenses(
                &file_io_spy,
                &Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                },
                &[
                    "example-COPYRIGHT".to_string(),
                    "example-LICENSE-APACHE".to_string()
                ]
            )
            .unwrap()
        );
    }
}
