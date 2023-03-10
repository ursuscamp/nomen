use std::{path::PathBuf, str::FromStr};

use anyhow::anyhow;
use bitcoin::{
    hashes::hex::ToHex, Address, OutPoint, PackedLockTime, Script, Sequence, Transaction, TxIn,
    TxOut, Txid, Witness,
};
use bitcoincore_rpc::{RawTx, RpcApi};
use serde::{Deserialize, Serialize};
use yansi::Paint;

use crate::{
    config::Config,
    documents::{self, ExampleDocument},
    name::{self, Name},
    pubkey::Pubkey,
};

pub fn example_create() -> anyhow::Result<()> {
    let doc = documents::Create::create_example();
    let doc = serde_json::to_string_pretty(&doc)?;
    println!("{doc}");
    Ok(())
}

/// (FQDN, Pubkey, Sub-Children)
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct ChildName((String, String, Vec<ChildName>));

pub fn create_new_tx(config: &Config, document: &PathBuf) -> anyhow::Result<()> {
    let document: documents::Create = serde_json::from_str(&std::fs::read_to_string(document)?)?;

    let name = get_valid_name(&document.pubkey, &document.name)?;
    let nsid = name.namespace_id();

    let (txid, vout, address) = coerce_inputs(&document.txid, document.vout, &document.address)?;

    let (txin, new_txout) = input_and_new_output(config, txid, vout, address)?;

    let op_out = op_return_output(name);

    let mut new_tx = create_transaction(txin, new_txout, op_out);

    let fee = calculate_fee(&new_tx, &document.fee_rate);
    new_tx.output[0].value -= fee; // Adjust transaction for estimated fee

    println!("Namespace ID: {}", Paint::green(nsid));
    println!(
        "{}",
        Paint::green(
            "Here is the unsigned tranasction. Sign it and broadcast it from your wallet:\n"
        )
    );
    println!("{}", new_tx.raw_hex());
    Ok(())
}

fn create_transaction(txin: TxIn, new_txout: TxOut, op_out: TxOut) -> Transaction {
    let mut new_tx = Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![txin],
        output: vec![new_txout, op_out],
    };
    new_tx
}

fn coerce_inputs(
    txid: &String,
    vout: u64,
    address: &String,
) -> Result<(Txid, usize, Address), anyhow::Error> {
    let txid: Txid = txid.parse()?;
    let address = Address::from_str(&address)?;
    Ok((txid, vout as usize, address))
}

fn input_and_new_output(
    config: &Config,
    txid: Txid,
    vout: usize,
    address: Address,
) -> Result<(TxIn, TxOut), anyhow::Error> {
    let client = config.rpc_client()?;
    let tx = client.get_raw_transaction(&txid, None)?;
    let txout = tx.output.get(vout).ok_or(anyhow!("Invalid output"))?;
    let txin = TxIn {
        previous_output: OutPoint {
            txid,
            vout: vout as u32,
        },
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };
    let new_txout = TxOut {
        value: txout.value,
        script_pubkey: address.script_pubkey(),
    };
    Ok((txin, new_txout))
}

fn calculate_fee(new_tx: &Transaction, fee_rate: &usize) -> u64 {
    // This signuature is unsigned, so let's add in in some extra bytes to account for possible sig
    let vsize = new_tx.vsize() + (64 / 4);
    let fee = (vsize * fee_rate) as u64;
    log::debug!("Estimated fee: {fee}");
    fee
}

fn op_return_output(name: Name) -> TxOut {
    let mut op_return = format!("ind\x00\x00").as_bytes().to_vec();
    let namespace_id = name.namespace_id();
    log::debug!("Namespace id for {}: {}", name.0, namespace_id.to_hex());
    op_return.extend(name.namespace_id().as_ref());
    let op_return = Script::new_op_return(&op_return);
    let op_out = TxOut {
        value: 0,
        script_pubkey: op_return,
    };
    op_out
}

fn get_valid_name(pubkey: &String, name: &String) -> anyhow::Result<Name> {
    Ok(Name(
        name.clone(),
        Pubkey::from_str(pubkey.as_ref())?,
        vec![],
    ))
}
