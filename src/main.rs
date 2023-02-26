#![allow(unused)]

use std::str::FromStr;

use bitcoin::{
    hashes::hex::FromHex, psbt::PartiallySignedTransaction, Address, OutPoint, PackedLockTime,
    Script, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

static AUTH: &'static str = include_str!("../.auth");

static TXID: &'static str = "473cbaaa0ac29a07a3ac14a46effc3df9c341083417b78378e0ae58d7daf6ffb";
static INPUT_VALUE: u64 = (50 * 100_000_000);
static OUTPUT_VALUE: u64 = INPUT_VALUE - 1000;
static VOUT: u32 = 0;
static ADDR: &'static str = "bcrt1qjydp0w246juddlpqsdzn7st43uhtg69ue5clv3";

fn main() {
    let outpoint = OutPoint::new(Txid::from_hex(TXID).unwrap(), VOUT);

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::default(),
    };

    let txout = TxOut {
        value: OUTPUT_VALUE,
        script_pubkey: Address::from_str(ADDR).unwrap().script_pubkey(),
    };

    let tx = Transaction {
        version: 2,
        lock_time: PackedLockTime::ZERO,
        input: vec![txin],
        output: vec![txout],
    };

    let psbt = PartiallySignedTransaction::from_unsigned_tx(tx).unwrap();
    println!("Psbt: {psbt}");
}

fn auth() -> (&'static str, Auth) {
    let mut lines = AUTH.lines();
    let ip = lines.next().unwrap();
    let auth = Auth::UserPass(lines.next().unwrap().into(), lines.next().unwrap().into());
    (ip, auth)
}
