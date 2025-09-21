use crate::cargo_metadata::Package;
use crate::collect_licenses::collect_licenses;
use crate::file_io::FileIO;
use crate::license::License;
use crate::validate_licenses::{LicenseStatus, validate_licenses};

pub fn check_licenses(file_io: impl FileIO, filtered_packages: Vec<Package>) -> anyhow::Result<()> {
    let mut issues_found = false;

    for package in filtered_packages {
        let licenses = collect_licenses(&file_io, &package)?;
        match validate_licenses(
            &file_io,
            &package.license.as_deref().map(License::parse),
            &licenses,
        ) {
            LicenseStatus::Valid => continue,
            status => {
                issues_found = true;
                status.warn(&package)
            }
        }
    }
    if issues_found {
        Err(anyhow::anyhow!("licenses had inconsistencies"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cargo_metadata::Package;
    use crate::check_licenses::check_licenses;
    use crate::file_io::FileIOSpy;

    #[test]
    fn failure_to_read_dir_causes_error() {
        let file_io_spy = FileIOSpy::default();
        file_io_spy
            .read_dir
            .returns
            .push_back(Err(anyhow::anyhow!("deliberate test error")));
        assert!(
            check_licenses(
                file_io_spy,
                vec![Package {
                    normalised_name: "example".to_string(),
                    path: Default::default(),
                    url: None,
                    license: None,
                }]
            )
            .is_err()
        );
    }
}
