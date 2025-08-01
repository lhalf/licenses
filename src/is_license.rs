use crate::file_io::DirEntry;

pub fn is_license(dir_entry: &DirEntry) -> bool {
    if !dir_entry.is_file {
        return false;
    }

    let filename = dir_entry.name.to_string_lossy().to_lowercase();

    ["license", "copying", "copyright"]
        .iter()
        .any(|license_name| filename.contains(license_name))
}

#[cfg(test)]
mod tests {
    use crate::file_io::DirEntry;
    use crate::is_license::is_license;
    use std::ffi::OsString;

    #[test]
    fn directories_are_not_licenses() {
        assert!(!is_license(&DirEntry {
            name: OsString::from("LICENSE_DIRECTORY"),
            path: Default::default(),
            is_file: false,
        }))
    }

    #[test]
    fn license_file_with_valid_name() {
        for license in [
            "LICENSE_APACHE",
            "LICENSE_MIT",
            "LICENSE",
            "license",
            "licenseother",
            "UNLICENSE",
            "COPYING",
            "COPYRIGHT",
        ] {
            assert!(is_license(&DirEntry {
                name: OsString::from(license),
                path: Default::default(),
                is_file: true
            }));
        }
    }

    #[test]
    fn license_file_with_invalid_name() {
        for license in ["LICENS_APACHE", "PATENT", "README"] {
            assert!(!is_license(&DirEntry {
                name: OsString::from(license),
                path: Default::default(),
                is_file: true
            }));
        }
    }
}
