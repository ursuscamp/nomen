mod config;
mod db;
mod subcommands;

use anyhow::bail;
use clap::Parser;

use config::Config;
use nomen_core::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = parse_config()?;

    let pool = db::initialize(&config).await?;

    match &config.cli.subcommand {
        config::Subcommand::Init => subcommands::init()?,
        config::Subcommand::Index => subcommands::index(&config).await?,
        config::Subcommand::Server => subcommands::start(&config, &pool).await?,
    }

    Ok(())
}

fn parse_config() -> anyhow::Result<Config> {
    let cli = config::Cli::parse();

    let file = if cli.config.is_file() {
        let config_str = std::fs::read_to_string(&cli.config)?;

        toml::from_str(&config_str)?
    } else {
        log::error!("Config file not found.");
        bail!("Missing config file.")
    };

    let config = Config::new(cli, file);

    log::debug!("Config loaded: {config:?}");

    Ok(config)
}
