use std::collections::HashMap;

use bitcoin::{
    psbt::{Output, Psbt},
    script::PushBytesBuf,
    ScriptBuf, TxOut,
};
use nomen_core::{CreateBuilder, NameKind, NsidBuilder};
use nostr_sdk::{EventBuilder, Tag, TagKind, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

pub(crate) fn insert_outputs(
    psbt: &mut Psbt,
    pubkey: &XOnlyPublicKey,
    name: &str,
) -> anyhow::Result<()> {
    let raw_script = CreateBuilder::new(pubkey, name).v1_op_return();
    let op_return: PushBytesBuf = raw_script.try_into()?;
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
