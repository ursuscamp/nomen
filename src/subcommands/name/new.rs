use std::io::Write;

use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, secp256k1::SecretKey};
use bitcoincore_rpc::{RawTx, RpcApi};
use itertools::Itertools;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Config, NameNewSubcommand, TxInfo},
    subcommands::name::{create_unsigned_tx, get_keys},
    util::{ChildPair, NameKind, Nsid, NsidBuilder},
    util::{Hash160, IndigoKind},
};

use super::{get_transaction, op_return};

pub async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let nsid = args
        .children
        .iter()
        .cloned()
        .map(ChildPair::pair)
        .fold(
            NsidBuilder::new(&args.name, &keys.public_key()),
            |acc, (n, pk)| acc.update_child(&n, pk),
        )
        .finalize();
    let tx = create_unsigned_tx(config, &args.txinfo, nsid, IndigoKind::Create).await?;

    println!("Nsid: {}", nsid.to_hex());
    println!("Unsigned Tx: {}", tx.raw_hex());

    let event = create_event(&args.children, nsid, args, keys)?;
    let (_k, nostr) = config.nostr_random_client().await?;
    let event_id = nostr.send_event(event).await?;

    println!("Sent event {event_id}");

    Ok(())
}

fn create_event(
    children: &[ChildPair],
    nsid: Nsid,
    args: &NameNewSubcommand,
    keys: Keys,
) -> Result<nostr_sdk::Event, anyhow::Error> {
    let children_json = {
        let s = children
            .iter()
            .cloned()
            .map(ChildPair::pair)
            .map(|(name, pubkey)| (name, pubkey.to_hex()))
            .collect_vec();
        serde_json::to_string(&s)
    }?;
    let event = EventBuilder::new(
        NameKind::Name.into(),
        children_json,
        &[
            Tag::Identifier(nsid.to_hex()),
            Tag::Generic(TagKind::Custom("ind".to_owned()), vec![args.name.clone()]),
        ],
    )
    .to_event(&keys)?;
    Ok(event)
}
