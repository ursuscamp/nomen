#![allow(unused)]

mod args;
mod hash160;
mod name;

use std::{borrow::BorrowMut, str::FromStr};

use args::Args;
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
    let mut args = Args::parse();

    if let Some(config) = &args.config {
        let config = std::fs::read_to_string(config)?;
        let config: Args = toml::from_str(&config)?;
        args = config.merge(&args);
    }

    println!("{args:#?}");

    Ok(())
}
