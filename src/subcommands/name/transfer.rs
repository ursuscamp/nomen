use bitcoincore_rpc::RawTx;
use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

use crate::{
    config::{Cli, Config, NameTransferSubcommand},
    util::{check_name, tag_print, Hash160, NameKind, NomenKind, Nsid, NsidBuilder},
};

#[derive(serde::Serialize)]
struct CmdOutput {
    nsid: String,
    unsigned_tx: String,
    event: String,
}

pub async fn transfer(config: &Config, args: &NameTransferSubcommand) -> anyhow::Result<()> {
    let name = args.name.as_ref();
    check_name(config, name).await?;
    let mut psbt = super::parse_psbt(&args.psbt)?;
    let keys = super::get_keys(&args.privkey)?;
    let nsid = NsidBuilder::new(name, &args.pubkey).finalize();
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();

    super::insert_outputs(&mut psbt, fingerprint, nsid, NomenKind::Transfer)?;

    let event = create_event(nsid, &keys, args)?;
    if args.broadcast {
        let (_k, nostr) = config.nostr_random_client().await?;
        nostr.send_event(event.clone()).await?;
    }

    let output = CmdOutput {
        nsid: nsid.to_string(),
        unsigned_tx: psbt.to_string(),
        event: serde_json::to_string(&event)?,
    };

    if args.json {
        println!("{}", serde_json::to_string(&output)?);
    } else {
        tag_print("Nsid", &output.nsid);
        tag_print("Unsigned Tx", &output.unsigned_tx);
        tag_print("Event", &output.event);
    }

    Ok(())
}

fn create_event(
    nsid: Nsid,
    keys: &Keys,
    args: &NameTransferSubcommand,
) -> Result<nostr_sdk::Event, anyhow::Error> {
    let event = EventBuilder::new(
        NameKind::Transfer.into(),
        args.pubkey.to_string(),
        &[
            Tag::Identifier(nsid.to_string()),
            Tag::Generic(
                TagKind::Custom("nom".to_owned()),
                vec![args.name.to_string()],
            ),
        ],
    )
    .to_event(keys)?;
    Ok(event)
}
