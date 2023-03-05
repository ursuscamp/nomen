#![allow(unused)]

mod config;
mod hash160;
mod name;
mod subcommands;

use std::{borrow::BorrowMut, str::FromStr};

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
        config::Subcommand::NewNameTx {
            name,
            input,
            address,
            pubkey,
            fee_rate,
        } => {
            subcommands::create_new_tx(&cli, name, input, address, pubkey, fee_rate)?;
        }
        config::Subcommand::GenerateKeypair => subcommands::generate_keypair(),
    }

    Ok(())
}
