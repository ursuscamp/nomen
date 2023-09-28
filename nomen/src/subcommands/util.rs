use std::collections::HashMap;

use anyhow::bail;
use bitcoin::{
    psbt::{Output, Psbt},
    script::PushBytesBuf,
    ScriptBuf, TxOut,
};
use nomen_core::util::{NameKind, NomenKind, Nsid, NsidBuilder};
use nostr_sdk::{EventBuilder, Tag, TagKind, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

use crate::{config::Config, db};

pub async fn check_name_availability(config: &Config, name: &str) -> anyhow::Result<()> {
    let conn = config.sqlite().await?;
    let available = db::name_available(&conn, name).await?;
    if !available {
        bail!("Name {name} already exists");
    }
    Ok(())
}
pub(crate) fn insert_outputs(
    psbt: &mut Psbt,
    fingerprint: [u8; 5],
    nsid: Nsid,
    kind: NomenKind,
) -> anyhow::Result<()> {
    let op_return: PushBytesBuf = op_return_v0(fingerprint, nsid, kind).try_into()?;
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

pub fn op_return_v0(fingerprint: [u8; 5], nsid: Nsid, kind: NomenKind) -> Vec<u8> {
    let mut v = Vec::with_capacity(25);
    v.extend(b"NOM\x00");
    v.push(kind.into());
    v.extend(fingerprint);
    v.extend(nsid.as_ref());
    v
}

pub fn op_return_v1(pubkey: XOnlyPublicKey, name: &str, kind: NomenKind) -> Vec<u8> {
    let mut v = Vec::with_capacity(80);
    v.extend(b"NOM\x01");
    v.push(kind.into());
    v.extend(pubkey.serialize());
    v.extend(name.as_bytes());
    v
}

pub fn name_event(
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
