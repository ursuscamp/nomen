use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, Transaction};
use bitcoincore_rpc::RawTx;
use itertools::Itertools;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Config, NameUpdateSubcommand, TxInfo},
    util::{ChildPair, IndigoKind, NameKind, Nsid, NsidBuilder},
};

use super::{create_unsigned_tx, get_keys, get_transaction, op_return};

pub async fn update(config: &Config, args: &NameUpdateSubcommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let nsid = args
        .children
        .iter()
        .fold(
            NsidBuilder::new(&args.name, &keys.public_key()),
            |acc, child| {
                let p = child.clone().pair();
                acc.update_child(&p.0, p.1)
            },
        )
        .prev(args.previous)
        .finalize();
    let tx = create_unsigned_tx(config, &args.txinfo, nsid, IndigoKind::Update).await?;

    println!("Nsid: {}", nsid.to_hex());
    println!("Unsigned Tx: {}", tx.raw_hex());

    let event = update_event(&args.children, args.previous, nsid, args, keys)?;
    let (_k, nostr) = config.nostr_random_client().await?;
    let event_id = nostr.send_event(event).await?;

    println!("Sent event {event_id}");

    Ok(())
}

fn update_event(
    children: &[ChildPair],
    prev: Nsid,
    nsid: Nsid,
    args: &NameUpdateSubcommand,
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
        NameKind::Update.into(),
        children_json,
        &[
            Tag::Identifier(nsid.to_hex()),
            Tag::Generic(
                TagKind::Custom("ind".to_owned()),
                vec![args.name.clone(), prev.to_hex()],
            ),
        ],
    )
    .to_event(&keys)?;
    Ok(event)
}
