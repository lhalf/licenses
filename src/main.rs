mod cargo_tree;
mod dependency;

fn main() -> Result<(), anyhow::Error> {
    let dependencies = cargo_tree::get_dependencies(1)?;
    dbg!(dependencies);
    Ok(())
}
