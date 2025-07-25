use crate::file_io::DirEntry;

pub fn is_license(dir_entry: &DirEntry) -> bool {
    dir_entry
        .name
        .to_string_lossy()
        .to_lowercase()
        .starts_with("license")
}

#[cfg(test)]
mod tests {
    use crate::file_io::DirEntry;
    use crate::is_license::is_license;
    use std::ffi::OsString;

    #[test]
    fn valid_license() {
        for license in [
            "LICENSE_APACHE",
            "LICENSE_MIT",
            "LICENSE",
            "license",
            "licenseother",
        ] {
            assert!(is_license(&DirEntry {
                name: OsString::from(license),
                path: Default::default(),
            }));
        }
    }

    #[test]
    fn invalid_license() {
        for license in ["LICENS_APACHE", "COPYING", "COPYRIGHT", "PATENT", "README"] {
            assert!(!is_license(&DirEntry {
                name: OsString::from(license),
                path: Default::default(),
            }));
        }
    }
}
