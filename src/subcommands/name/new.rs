use std::collections::HashMap;

use bitcoin::hashes::hex::ToHex;
use bitcoincore_rpc::RawTx;

use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Config, NameNewSubcommand},
    subcommands::name::{create_unsigned_tx, get_keys},
    util::{tag_print, Hash160, NameKind, NomenKind, Nsid, NsidBuilder},
};

#[derive(serde::Serialize)]
struct CmdOutput {
    nsid: String,
    unsigned_tx: String,
    event_id: String,
}

pub async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let nsid = NsidBuilder::new(&args.name, &keys.public_key()).finalize();
    let fingerprint = Hash160::default()
        .chain_update(args.name.as_bytes())
        .fingerprint();
    let tx = create_unsigned_tx(config, &args.txinfo, fingerprint, nsid, NomenKind::Create).await?;

    let event = super::name_event(keys.public_key(), &HashMap::new(), &args.name)?.sign(&keys)?;
    let (_k, nostr) = config.nostr_random_client().await?;
    let event_id = nostr.send_event(event).await?;

    let output = CmdOutput {
        nsid: nsid.to_hex(),
        unsigned_tx: tx.raw_hex(),
        event_id: event_id.to_string(),
    };

    if args.json {
        println!("{}", serde_json::to_string(&output)?);
    } else {
        tag_print("Nsid", &output.nsid);
        tag_print("Unsigned Tx", &output.unsigned_tx);
        tag_print("Event ID broadcast", &output.event_id);
    }

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
            Tag::Generic(TagKind::Custom("nom".to_owned()), vec![args.name.clone()]),
        ],
    )
    .to_event(&keys)?;
    Ok(event)
}
