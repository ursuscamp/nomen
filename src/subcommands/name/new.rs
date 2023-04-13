use std::io::Write;

use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, secp256k1::SecretKey};
use bitcoincore_rpc::{RawTx, RpcApi};
use itertools::Itertools;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Config, NameNewSubcommand, TxInfo},
    subcommands::name::{create_unsigned_tx, get_keys},
    util::{Hash160, IndigoKind},
    util::{NameKind, Nsid, NsidBuilder},
};

use super::{get_transaction, op_return};

pub async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let nsid = NsidBuilder::new(&args.name, &keys.public_key()).finalize();
    let tx = create_unsigned_tx(config, &args.txinfo, nsid, IndigoKind::Create).await?;

    println!("Nsid: {}", nsid.to_hex());
    println!("Unsigned Tx: {}", tx.raw_hex());

    let event = create_event(nsid, args, keys)?;
    let (_k, nostr) = config.nostr_random_client().await?;
    let event_id = nostr.send_event(event).await?;

    println!("Sent event {event_id}");

    Ok(())
}

fn create_event(
    nsid: Nsid,
    args: &NameNewSubcommand,
    keys: Keys,
) -> Result<nostr_sdk::Event, anyhow::Error> {
    let event = EventBuilder::new(
        NameKind::Name.into(),
        "",
        &[
            Tag::Identifier(nsid.to_hex()),
            Tag::Generic(TagKind::Custom("ind".to_owned()), vec![args.name.clone()]),
        ],
    )
    .to_event(&keys)?;
    Ok(event)
}
