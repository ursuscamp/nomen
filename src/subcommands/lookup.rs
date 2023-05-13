use yansi::Paint;

use crate::{config::Config, util::check_name};

pub async fn lookup(config: &Config, name: &str) -> anyhow::Result<()> {
    let name = name.to_lowercase();
    let (name, msg) = match check_name(config, &name).await {
        Ok(_) => (Paint::yellow(&name), Paint::green("available")),
        Err(_) => (Paint::yellow(&name), Paint::red("unavailable")),
    };

    println!("Name {name} is {msg}.");
    Ok(())
}
