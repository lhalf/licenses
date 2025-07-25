use crate::cargo_metadata::Package;
use itertools::Itertools;
use std::collections::BTreeSet;

pub fn summarise(crates: BTreeSet<String>, all_packages: Vec<Package>) {
    all_packages
        .into_iter()
        .filter(|package| crates.contains(&package.normalised_name))
        .filter_map(|package| package.license)
        .unique()
        .sorted()
        .for_each(|license| println!("{license}"))
}
