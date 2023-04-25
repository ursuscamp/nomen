mod new;
mod record;
mod transfer;

use std::{collections::HashMap, io::Write};

pub use anyhow::anyhow;
use bitcoin::{secp256k1::SecretKey, XOnlyPublicKey};
use bitcoincore_rpc::RpcApi;
pub use new::*;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag, UnsignedEvent};
pub use record::*;

use crate::{
    config::{Config, NameSubcommand, TxInfo},
    util::{NameKind, NomenKind, Nsid, NsidBuilder},
};

pub async fn name(config: &Config, cmd: &NameSubcommand) -> anyhow::Result<()> {
    match cmd {
        NameSubcommand::New(new_data) => new::new(config, new_data).await?,
        NameSubcommand::Record(record_data) => record::record(config, record_data).await?,
        NameSubcommand::Transfer(transfer_data) => {
            transfer::transfer(config, transfer_data).await?
        }
    }

    Ok(())
}

pub(crate) fn get_keys(privkey: &Option<SecretKey>) -> Result<Keys, anyhow::Error> {
    let privkey = if let Some(s) = privkey {
        *s
    } else {
        // TODO: use a better system for getting secure info than this, like a secure prompt
        print!("Private key: ");
        std::io::stdout().flush()?;
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        s.trim().to_string().parse()?
    };
    let keys = Keys::new(privkey);
    Ok(keys)
}

pub(crate) async fn get_transaction(
    config: &Config,
    txid: &bitcoin::Txid,
) -> Result<bitcoin::Transaction, anyhow::Error> {
    let client = config.rpc_client()?;
    let txid = *txid;
    Ok(tokio::task::spawn_blocking(move || client.get_raw_transaction(&txid, None)).await??)
}

pub(crate) fn op_return(fingerprint: [u8; 5], nsid: Nsid, kind: NomenKind) -> Vec<u8> {
    let mut v = Vec::with_capacity(25);
    v.extend(b"NOM\x00");
    v.push(kind.into());
    v.extend(fingerprint);
    v.extend(nsid.as_ref());
    v
}

pub(crate) async fn create_unsigned_tx(
    config: &Config,
    args: &TxInfo,
    fingerprint: [u8; 5],
    nsid: Nsid,
    kind: NomenKind,
) -> Result<bitcoin::Transaction, anyhow::Error> {
    let tx = get_transaction(config, &args.txid).await?;
    let txout = &tx.output[args.vout as usize];
    let new_amount = txout
        .value
        .checked_sub(args.fee as u64)
        .ok_or_else(|| anyhow!("Fee is over available amount in tx"))?;
    let txin = bitcoin::TxIn {
        previous_output: bitcoin::OutPoint {
            txid: args.txid,
            vout: args.vout,
        },
        script_sig: bitcoin::Script::new(), // Unsigned tx with empty script
        sequence: bitcoin::Sequence::ZERO,
        witness: bitcoin::Witness::new(),
    };
    let txout = bitcoin::TxOut {
        value: new_amount,
        script_pubkey: args.address.script_pubkey(),
    };
    let op_return = bitcoin::TxOut {
        value: 0,
        script_pubkey: bitcoin::Script::new_op_return(&op_return(fingerprint, nsid, kind)),
    };
    let tx = bitcoin::Transaction {
        version: 1,
        lock_time: bitcoin::PackedLockTime::ZERO,
        input: vec![txin],
        output: vec![txout, op_return],
    };
    Ok(tx)
}

pub(crate) fn name_event(
    pubkey: XOnlyPublicKey,
    records: &HashMap<String, String>,
    name: &str,
) -> anyhow::Result<UnsignedEvent> {
    let records = serde_json::to_string(&records)?;
    let nsid = NsidBuilder::new(name, &pubkey).finalize();
    let event = EventBuilder::new(
        NameKind::Name.into(),
        records,
        &[
            Tag::Identifier(nsid.to_string()),
            Tag::Generic(TagKind::Custom("nom".to_owned()), vec![name.to_owned()]),
        ],
    )
    .to_unsigned_event(pubkey);

    Ok(event)
}
