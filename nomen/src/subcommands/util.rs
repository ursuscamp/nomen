use std::collections::HashMap;

use anyhow::anyhow;
use bitcoin::{
    psbt::{Output, Psbt},
    script::PushBytesBuf,
    Amount, ScriptBuf, TxOut, Txid,
};
use bitcoincore_rpc::{bitcoincore_rpc_json::CreateRawTransactionInput, Client, RpcApi};
use nomen_core::{CreateBuilder, NameKind, NsidBuilder, TransferBuilder};
use nostr_sdk::{EventBuilder, Tag, TagKind, UnsignedEvent};
use secp256k1::{schnorr::Signature, XOnlyPublicKey};

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

#[allow(clippy::unused_async)]
pub async fn transfer_psbt1(
    client: Client,
    txid: Txid,
    vout: u32,
    address: &str,
    name: &str,
    pubkey: &XOnlyPublicKey,
    fee_rate: usize,
) -> anyhow::Result<Psbt> {
    let op_return = transfer_op_return(pubkey, name);
    create_transaction(
        client,
        txid,
        vout,
        address.to_string(),
        op_return,
        fee_rate,
        None,
    )
    .await
}

#[allow(clippy::unused_async, clippy::too_many_arguments)]
pub async fn transfer_psbt2(
    client: Client,
    txid: Txid,
    vout: u32,
    address: String,
    name: String,
    pubkey: XOnlyPublicKey,
    fee_rate: usize,
    sig: Signature,
    value: Amount,
) -> anyhow::Result<Psbt> {
    let op_return = signature_op_return(&pubkey, &name, sig);
    create_transaction(
        client,
        txid,
        vout,
        address,
        op_return,
        fee_rate,
        Some(value),
    )
    .await
}

async fn create_transaction(
    client: Client,
    txid: Txid,
    vout: u32,
    address: String,
    op_return: TxOut,
    fee_rate: usize,
    value: Option<Amount>,
) -> Result<Psbt, anyhow::Error> {
    tokio::task::spawn_blocking(move || -> Result<Psbt, anyhow::Error> {
        // Get UTXO info from the Bitcoin Node, then construct a new transaction with the specified inputs and outputs, plus the name OP_RETURN
        let (scriptpk, value) = if let Some(amount) = value {
            (ScriptBuf::new(), amount)
        } else {
            let utxo = client
                .get_tx_out(&txid, vout, Some(false))?
                .ok_or(anyhow!("Tx not found"))?;
            (
                utxo.script_pub_key
                    .address
                    .ok_or_else(|| anyhow!("Invalid prev out"))
                    .map(|f| f.assume_checked().script_pubkey())
                    .unwrap_or_default(),
                utxo.value,
            )
        };
        let input = CreateRawTransactionInput {
            txid,
            vout,
            sequence: None,
        };
        let mut outputs = HashMap::new();
        outputs.insert(address.to_string(), value);
        let mut tx = client.create_raw_transaction(&[input], &outputs, None, Some(true))?;
        tx.output.push(op_return);
        let size = tx.vsize();
        let fee = size * fee_rate;
        tx.output[0].value -= fee as u64;
        let mut psbt = Psbt::from_unsigned_tx(tx)?;
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: value.to_sat(),
            script_pubkey: scriptpk,
        });
        // psbt.outputs = psbt.unsigned_tx.output.clone();
        Ok(psbt)
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

fn transfer_op_return(new_owner: &XOnlyPublicKey, name: &str) -> TxOut {
    let tb = TransferBuilder {
        new_pubkey: new_owner,
        name,
    };
    let or = tb.transfer_op_return();
    let pb = PushBytesBuf::try_from(or).expect("invalid OP_RETURN");
    let script = ScriptBuf::new_op_return(&pb);
    TxOut {
        value: 0,
        script_pubkey: script,
    }
}

fn signature_op_return(new_pubkey: &XOnlyPublicKey, name: &str, sig: Signature) -> TxOut {
    let tb = TransferBuilder { new_pubkey, name };
    let or = tb.signature_provided_op_return(sig);
    let pb = PushBytesBuf::try_from(or).expect("invalid OP_RETURN");
    let script = ScriptBuf::new_op_return(&pb);

    TxOut {
        value: 0,
        script_pubkey: script,
    }
}
