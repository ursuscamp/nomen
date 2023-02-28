#![allow(unused)]

mod name;

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

static AUTH: &'static str = include_str!("../.auth");

static TXID: &'static str = "579807220e5759bb738c067ba279dd886b167f9a505242bf1329f57e677ce605";
static INPUT_VALUE: u64 = (50 * 100_000_000);
static OUTPUT_VALUE: u64 = INPUT_VALUE - 1000;
static VOUT: u32 = 0;
static ADDR: &'static str = "bcrt1qqssfvug98rf6f668a2x6rr47dxdhymrrpt4z07";

fn main() {
    // let script = Builder::new()
    //     .push_opcode(OP_FALSE)
    //     .push_opcode(OP_IF)
    //     .push_slice("hello world".as_bytes())
    //     .push_opcode(OP_ENDIF)
    //     .into_script();

    let script = Script::new_op_return("hello_world".as_bytes());

    let outpoint = OutPoint::new(Txid::from_hex(TXID).unwrap(), VOUT);

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let txout = TxOut {
        value: OUTPUT_VALUE,
        script_pubkey: Address::from_str(ADDR).unwrap().script_pubkey(),
    };

    let op_return = TxOut {
        value: 0,
        script_pubkey: script,
    };

    let tx = Transaction {
        version: 2,
        lock_time: PackedLockTime::ZERO,
        input: vec![txin],
        output: vec![txout, op_return],
    };

    let txh = tx.raw_hex();
    println!("Transaction: {txh}");
}

fn auth() -> (&'static str, Auth) {
    let mut lines = AUTH.lines();
    let ip = lines.next().unwrap();
    let auth = Auth::UserPass(lines.next().unwrap().into(), lines.next().unwrap().into());
    (ip, auth)
}
