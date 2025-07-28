use crate::cargo_metadata::Package;
use itertools::Itertools;

pub fn summarise(filtered_packages: Vec<Package>) {
    filtered_packages
        .into_iter()
        .filter_map(|package| package.license)
        .unique()
        .sorted()
        .for_each(|license| println!("{license}"))
}
