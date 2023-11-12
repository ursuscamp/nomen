#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

mod config;
mod db;
mod subcommands;
mod util;

use anyhow::bail;
use clap::Parser;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = parse_config()?;

    let pool = db::initialize(&config).await?;

    match &config.cli.subcommand {
        config::Subcommand::Init => subcommands::init()?,
        config::Subcommand::Index => subcommands::index(&config).await?,
        config::Subcommand::Server => subcommands::start(&config, &pool).await?,
        config::Subcommand::Reindex { blockheight } => {
            subcommands::reindex(&config, &pool, blockheight.unwrap_or_default()).await?;
        }
        config::Subcommand::Rescan { blockheight } => {
            subcommands::rescan(&config, &pool, blockheight.unwrap_or_default()).await?;
        }
        config::Subcommand::Version => {
            subcommands::version();
        }
    }

    Ok(())
}

fn parse_config() -> anyhow::Result<Config> {
    let cli = config::Cli::parse();

    let file = if cli.config.is_file() {
        let config_str = std::fs::read_to_string(&cli.config)?;

        toml::from_str(&config_str)?
    } else {
        tracing::error!("Config file not found.");
        bail!("Missing config file.")
    };

    let config = Config::new(cli, file);

    tracing::debug!("Config loaded: {config:?}");

    Ok(config)
}
