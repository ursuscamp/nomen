mod new;
mod record;

use std::{collections::HashMap, io::Write, path::PathBuf, str::FromStr};

pub use anyhow::anyhow;
use bitcoin::{
    psbt::{Output, Psbt},
    script::PushBytesBuf,
    secp256k1::SecretKey,
    ScriptBuf, TxOut,
};
use bitcoincore_rpc::RpcApi;
pub use new::*;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag, UnsignedEvent};
pub use record::*;
use secp256k1::XOnlyPublicKey;

use crate::{
    config::{Cli, Config, NameSubcommand, TxInfo},
    util::{NameKind, NomenKind, NostrSk, Nsid, NsidBuilder},
};

pub async fn name(config: &Config, cmd: &NameSubcommand) -> anyhow::Result<()> {
    match cmd {
        NameSubcommand::New(new_data) => new::new(config, new_data).await?,
        NameSubcommand::Record(record_data) => record::record(config, record_data).await?,
    }

    Ok(())
}

pub(crate) fn get_keys(privkey: &Option<NostrSk>) -> Result<Keys, anyhow::Error> {
    let privkey = if let Some(s) = privkey {
        *s.as_ref()
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

pub(crate) fn insert_outputs(
    psbt: &mut Psbt,
    fingerprint: [u8; 5],
    nsid: Nsid,
    kind: NomenKind,
) -> anyhow::Result<()> {
    let op_return: PushBytesBuf = super::op_return_v0(fingerprint, nsid, kind).try_into()?;
    let op_return = ScriptBuf::new_op_return(&op_return);
    psbt.unsigned_tx.output.push(TxOut {
        value: 0,
        script_pubkey: op_return.clone(),
    });
    psbt.outputs.push(Output {
        redeem_script: Some(op_return),
        ..Default::default()
    });

    Ok(())
}

pub(crate) async fn get_transaction(
    config: &Config,
    txid: &bitcoin::Txid,
) -> Result<bitcoin::Transaction, anyhow::Error> {
    let client = config.rpc_client()?;
    let txid = *txid;
    Ok(tokio::task::spawn_blocking(move || client.get_raw_transaction(&txid, None)).await??)
}

pub(crate) fn op_return_v0(fingerprint: [u8; 5], nsid: Nsid, kind: NomenKind) -> Vec<u8> {
    let mut v = Vec::with_capacity(25);
    v.extend(b"NOM\x00");
    v.push(kind.into());
    v.extend(fingerprint);
    v.extend(nsid.as_ref());
    v
}

pub(crate) fn op_return_v1(pubkey: XOnlyPublicKey, name: &str, kind: NomenKind) -> Vec<u8> {
    let mut v = Vec::with_capacity(80);
    v.extend(b"NOM\x01");
    v.push(kind.into());
    v.extend(pubkey.serialize());
    v.extend(name.as_bytes());
    v
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

pub(crate) fn parse_psbt(psbt: &str) -> anyhow::Result<Psbt> {
    Ok(match PathBuf::from_str(psbt) {
        Ok(path) if path.exists() => Psbt::deserialize(&std::fs::read(path)?)?,
        _ => Psbt::from_str(psbt)?,
    })
}
