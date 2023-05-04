#![allow(unused)]

mod config;
mod db;
mod subcommands;
mod util;

use clap::Parser;
use config::Config;

use crate::config::{Cli, ConfigFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = parse_config()?;

    let pool = db::initialize(&config).await?;

    match &config.cli.subcommand {
        config::Subcommand::Noop => {}
        config::Subcommand::GenerateKeypair => subcommands::generate_keypair(),
        config::Subcommand::SignEvent(event) => subcommands::sign_event(&config, event).await?,
        config::Subcommand::Name(name) => subcommands::name(&config, name).await?,
        config::Subcommand::Index => subcommands::index(&config, &pool).await?,
        config::Subcommand::Server(server) => subcommands::start(&config, &pool, server).await?,
        config::Subcommand::Debug(debug) => match debug {
            config::DebugSubcommand::ListNamespaces => subcommands::list_namespaces()?,
            config::DebugSubcommand::NamesIndex => subcommands::names_index()?,
        },
    }

    Ok(())
}

fn parse_config() -> anyhow::Result<Config> {
    let mut cli = config::Cli::parse();
    let config_name = cli.config.clone().unwrap_or_else(|| ".nomen.toml".into());

    let file = if config_name.is_file() {
        let config_str = std::fs::read_to_string(config_name)?;

        toml::from_str(&config_str)?
    } else {
        log::info!("Config file not found. Skipping.");
        ConfigFile::default()
    };

    let config = Config::new(cli, file);

    log::debug!("Config loaded: {config:?}");

    Ok(config)
}
