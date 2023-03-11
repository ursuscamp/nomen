use crate::documents::{self, ExampleDocument};

pub fn example_records() -> anyhow::Result<()> {
    let doc = serde_json::to_string_pretty(&documents::Records::create_example())?;
    println!("{doc}");

    Ok(())
}
