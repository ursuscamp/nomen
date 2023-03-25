#![allow(unused)]

mod config;
mod db;
mod hash160;
// mod name;
mod subcommands;
mod util;
mod validators;

use clap::Parser;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = parse_config()?;

    let pool = db::initialize(&config).await?;

    match &config.subcommand {
        config::Subcommand::Noop => {}
        config::Subcommand::GenerateKeypair => subcommands::generate_keypair(),
        config::Subcommand::Name(name) => subcommands::name(&config, name).await?,
        // config::Subcommand::Index(index) => match index {
        //     config::IndexSubcommand::Blockchain {
        //         confirmations,
        //         height,
        //     } => {
        //         subcommands::index_blockchain(&config, &pool, confirmations.unwrap_or(3), *height)
        //             .await?
        //     }
        //     config::IndexSubcommand::CreateEvents => {
        //         subcommands::index_create_events(&config, &pool).await?
        //     }
        //     config::IndexSubcommand::RecordsEvents => {
        //         subcommands::index_records_events(&config, &pool).await?
        //     }
        // },
        config::Subcommand::Index => subcommands::index(&config, &pool).await?,
        config::Subcommand::Server {
            bind: _,
            confirmations: _,
            height: _,
            without_explorer,
            without_api,
            without_indexer,
        } => {
            subcommands::start(
                &config,
                &pool,
                *without_explorer,
                *without_api,
                *without_indexer,
            )
            .await?
        }
        config::Subcommand::Debug(debug) => match debug {
            config::DebugSubcommand::ListNamespaces => subcommands::list_namespaces()?,
            config::DebugSubcommand::NamesIndex => subcommands::names_index()?,
        },
    }

    Ok(())
}

fn parse_config() -> anyhow::Result<Config> {
    let mut config = config::Config::parse();
    let config_name = config
        .config
        .clone()
        .unwrap_or_else(|| ".indigo.toml".into());

    if config_name.is_file() {
        let config_str = std::fs::read_to_string(config_name)?;
        let config_file = toml::from_str(&config_str)?;
        config.merge_config_file(config_file);
    } else {
        log::info!("Config file not found. Skipping.");
    }

    log::debug!("Config loaded: {config:?}");

    Ok(config)
}
