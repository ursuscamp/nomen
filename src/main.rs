#![allow(unused)]

mod config;
mod documents;
mod hash160;
mod name;
mod nostr;
mod subcommands;

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
use config::Config;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut cli = Config::parse();

    if let Some(config) = &cli.config {
        if config.is_file() {
            let config = std::fs::read_to_string(config)?;
            let config: Config = toml::from_str(&config)?;
            cli = cli.merge_config_file(&config);
        } else {
            log::info!("Config file not found. Skipping.");
        }
    }

    log::debug!("Config loaded: {cli:?}");

    match &cli.subcommand {
        config::Subcommand::Noop => {}
        config::Subcommand::GenerateKeypair => subcommands::generate_keypair(),
        config::Subcommand::New(new) => match new {
            config::NewSubcommand::Tx { document } => subcommands::create_new_tx(&cli, document)?,
            config::NewSubcommand::Broadcast {
                namespace_id,
                privkey,
            } => subcommands::broadcast_new_name(&cli, namespace_id, privkey)?,
            config::NewSubcommand::Example => subcommands::example_create()?,
        },
        config::Subcommand::Index => subcommands::index_blockchain(&cli)?,
    }

    Ok(())
}
