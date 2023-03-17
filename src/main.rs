#![allow(unused)]

mod config;
mod db;
mod documents;
mod hash160;
mod name;
mod subcommands;
mod util;
mod validators;

use std::{borrow::BorrowMut, path::PathBuf, str::FromStr};

use bitcoin::{
    blockdata::{
        opcodes::{
            all::{OP_ENDIF, OP_IF},
            OP_FALSE,
        },
        script::Builder,
    },
    hashes::hex::FromHex,
    psbt::{serialize::Deserialize, PartiallySignedTransaction, Psbt},
    Address, OutPoint, PackedLockTime, Script, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use bitcoincore_rpc::{Auth, Client, RawTx, RpcApi};
use clap::Parser;

use crate::config::{Config, ConfigFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = parse_config()?;

    db::initialize(&config).await?;

    match &config.subcommand {
        config::Subcommand::Noop => {}
        config::Subcommand::GenerateKeypair => subcommands::generate_keypair(),
        config::Subcommand::New(new) => match new {
            config::NewSubcommand::Tx { document } => {
                subcommands::create_new_tx(&config, &document)?
            }
            config::NewSubcommand::Broadcast { document, privkey } => {
                subcommands::broadcast_new_name(&config, &document, &privkey).await?
            }
            config::NewSubcommand::Example => subcommands::example_create()?,
        },
        config::Subcommand::Records(records) => match records {
            config::RecordsSubcommand::Broadcast { document, privkey } => {
                subcommands::broadcast_records(&config, document.as_ref(), privkey.as_str()).await?
            }
            config::RecordsSubcommand::Example => subcommands::example_records()?,
        },
        config::Subcommand::Index(index) => match index {
            config::IndexSubcommand::Blockchain {
                confirmations,
                height,
            } => {
                subcommands::index_blockchain(&config, confirmations.unwrap_or(3), *height).await?
            }
            config::IndexSubcommand::CreateEvents => {
                subcommands::index_create_events(&config).await?
            }
            config::IndexSubcommand::RecordsEvents => {
                subcommands::index_records_events(&config).await?
            }
        },
        config::Subcommand::Server {
            bind,
            confirmations,
            height,
        } => subcommands::start(&config).await?,
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
