mod index;
mod server;
pub mod util;

pub use index::*;
pub use server::*;

use crate::config::ConfigFile;

pub(crate) fn init() -> anyhow::Result<()> {
    let config_file = ConfigFile::example();
    let cfg = toml::to_string(&config_file)?;
    println!("{cfg} ");
    Ok(())
}
