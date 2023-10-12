use std::collections::HashMap;

use anyhow::anyhow;
use bitcoin::{psbt::Psbt, script::PushBytesBuf, ScriptBuf, TxOut, Txid};
use bitcoincore_rpc::{bitcoincore_rpc_json::CreateRawTransactionInput, Client, RpcApi};
use nomen_core::{CreateBuilder, NameKind, NsidBuilder};
use nostr_sdk::{EventBuilder, Tag, TagKind, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

pub async fn create_psbt(
    client: Client,
    txid: Txid,
    vout: u32,
    address: String,
    name: String,
    pubkey: XOnlyPublicKey,
    fee_rate: usize,
) -> anyhow::Result<String> {
    let op_return = new_name_op_return(pubkey, &name);
    tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
        // Get UTXO info from the Bitcoin Node, then construct a new transaction with the specified inputs and outputs, plus the name OP_RETURN
        let utxo = client
            .get_tx_out(&txid, vout, Some(false))?
            .ok_or(anyhow!("Tx not found"))?;
        let input = CreateRawTransactionInput {
            txid,
            vout,
            sequence: None,
        };
        let mut outputs = HashMap::new();
        outputs.insert(address.to_string(), utxo.value);
        let mut tx = client.create_raw_transaction(&[input], &outputs, None, Some(true))?;
        tx.output.push(op_return);
        let size = tx.vsize();
        let fee = size * fee_rate;
        tx.output[0].value -= fee as u64;
        let psbt = Psbt::from_unsigned_tx(tx)?;
        Ok(psbt.to_string())
    })
    .await?
}

fn new_name_op_return(pubkey: XOnlyPublicKey, name: &str) -> TxOut {
    let or = CreateBuilder::new(&pubkey, name).v1_op_return();
    let pb = PushBytesBuf::try_from(or).expect("invalid OP_RETURN");
    let script = ScriptBuf::new_op_return(&pb);
    TxOut {
        value: 0,
        script_pubkey: script,
    }
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
