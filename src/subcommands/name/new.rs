use bitcoin::hashes::hex::ToHex;
use bitcoincore_rpc::RawTx;

use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Config, NameNewSubcommand},
    subcommands::name::{create_unsigned_tx, get_keys},
    util::{NameKind, NomenKind, Nsid, NsidBuilder},
};

pub async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let nsid = NsidBuilder::new(&args.name, &keys.public_key()).finalize();
    let tx = create_unsigned_tx(config, &args.txinfo, nsid, NomenKind::Create).await?;

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
