#![allow(unused)]

mod args;
mod hash160;
mod name;

use std::{borrow::BorrowMut, str::FromStr};

use args::Cli;
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

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut cli = Cli::parse();

    if let Some(config) = &cli.config {
        if config.is_file() {
            let config = std::fs::read_to_string(config)?;
            let config: Cli = toml::from_str(&config)?;
            cli = config.merge(&cli);
        } else {
            log::info!("Config file not found. Skipping.");
        }
    }

    log::debug!("Config loaded: {cli:?}");

    Ok(())
}
