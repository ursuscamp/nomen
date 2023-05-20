use std::{collections::HashMap, path::PathBuf, str::FromStr};

use anyhow::bail;
use bitcoin::{
    absolute::LockTime,
    psbt::{self, Output, Psbt},
    script::PushBytesBuf,
    ScriptBuf, Transaction, TxOut,
};
use bitcoincore_rpc::{RawTx, RpcApi};

use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};
use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::{
    config::{Cli, Config, NameNewSubcommand},
    db::{self},
    subcommands::name::get_keys,
    util::{check_name, tag_print, Hash160, NameKind, NomenKind, Nsid, NsidBuilder},
};

#[derive(serde::Serialize)]
struct CmdOutput {
    nsid: String,
    unsigned_tx: String,
    event: String,
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
            Tag::Identifier(nsid.to_string()),
            Tag::Generic(
                TagKind::Custom("nom".to_owned()),
                vec![args.name.to_string()],
            ),
        ],
    )
    .to_event(&keys)?;
    Ok(event)
}

pub(crate) async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
    let name = args.name.as_ref();
    check_name(config, name).await?;
    let mut psbt = super::parse_psbt(&args.psbt)?;
    let keys = get_keys(&args.privkey)?;
    let nsid = NsidBuilder::new(name, &keys.public_key()).finalize();
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();

    super::insert_outputs(&mut psbt, fingerprint, nsid, NomenKind::Create)?;

    let event = super::name_event(keys.public_key(), &HashMap::new(), name)?.sign(&keys)?;
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
