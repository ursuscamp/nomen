use bitcoin::hashes::hex::ToHex;
use bitcoincore_rpc::RawTx;
use nostr_sdk::{prelude::TagKind, EventBuilder, Tag};

use crate::{
    config::{Config, NameTransferSubcommand},
    util::{tag_print, Hash160, NameKind, NomenKind, Nsid, NsidBuilder},
};

use super::create_unsigned_tx;

#[derive(serde::Serialize)]
struct CmdOutput {
    nsid: String,
    unsigned_tx: String,
    unsigned_event: String,
}

pub async fn transfer(config: &Config, args: &NameTransferSubcommand) -> anyhow::Result<()> {
    let name = args.name.as_ref();
    let nsid = NsidBuilder::new(name, &args.new).finalize();
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();
    let unsigned_tx =
        create_unsigned_tx(config, &args.txinfo, fingerprint, nsid, NomenKind::Transfer).await?;
    let unsigned_event = create_event(nsid, args)?;
    let output = CmdOutput {
        nsid: nsid.to_hex(),
        unsigned_tx: unsigned_tx.raw_hex(),
        unsigned_event: serde_json::to_string(&unsigned_event)?,
    };

    if args.json {
        println!("{}", serde_json::to_string(&output)?);
    } else {
        tag_print("Nsid", &output.nsid);
        tag_print("Unsigned Tx", &output.unsigned_tx);
        tag_print("Unsigned Event", &output.unsigned_event);
    }

    Ok(())
}

fn create_event(
    nsid: Nsid,
    args: &NameTransferSubcommand,
) -> Result<nostr_sdk::UnsignedEvent, anyhow::Error> {
    let event = EventBuilder::new(
        NameKind::Transfer.into(),
        args.new.to_hex(),
        &[
            Tag::Identifier(nsid.to_hex()),
            Tag::Generic(
                TagKind::Custom("nom".to_owned()),
                vec![args.name.to_string()],
            ),
        ],
    )
    .to_unsigned_event(args.previous);
    Ok(event)
}
