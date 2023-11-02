use std::collections::HashMap;

use bitcoin::{
    psbt::{Output, Psbt},
    script::PushBytesBuf,
    ScriptBuf, TxOut,
};
use nomen_core::{CreateBuilder, NameKind, NsidBuilder};
use nostr_sdk::{EventBuilder, Tag, TagKind, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

pub fn extend_psbt(psbt: &mut Psbt, name: &str, pubkey: &XOnlyPublicKey) {
    let data = CreateBuilder::new(pubkey, name).v1_op_return();
    let mut pb = PushBytesBuf::new();
    pb.extend_from_slice(&data).expect("OP_RETURN fail");
    let data = ScriptBuf::new_op_return(&pb);
    psbt.outputs.push(Output {
        witness_script: Some(data.clone()),
        ..Default::default()
    });
    psbt.unsigned_tx.output.push(TxOut {
        value: 0,
        script_pubkey: data,
    });
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
