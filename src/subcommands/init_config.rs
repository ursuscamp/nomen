use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::bail;

use crate::config::ConfigFile;

pub fn init_config(path: &Option<PathBuf>) -> anyhow::Result<()> {
    let file = path.clone().unwrap_or_else(|| ".nomen.toml".into());
    if file.exists() {
        bail!("Config file already exists.");
    }

    let mut file = std::fs::File::create(&file)?;
    let config_file = ConfigFile::init();

    let strout = toml::to_string_pretty(&config_file)?;
    file.write_all(strout.as_bytes())?;
    Ok(())
}
