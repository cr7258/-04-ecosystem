use anyhow::Result;
use derive_builder::Builder;

#[allow(unused)]
#[derive(Builder, Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice".to_string())
        .age(30)
        .build()?;

    println!("{:?}", user);

    Ok(())
}
