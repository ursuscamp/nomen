use std::{ops::Deref, path::Path, str::FromStr};

use bitcoin::hashes::hex::ToHex;
use nostr_sdk::{
    prelude::{FromSkStr, TagKind},
    Event, EventBuilder, EventId, Keys, Tag, Timestamp,
};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config::Config, documents::Create, util::NamespaceNostrKind};

pub async fn broadcast_new_name(
    config: &Config,
    document: &Path,
    privkey: &String,
) -> anyhow::Result<()> {
    let create: Create = serde_json::from_str(&std::fs::read_to_string(document)?)?;
    let keys = Keys::from_sk_str(privkey.as_ref())?;
    let event = new_event(&create, privkey)?;

    let (_keys, client) = config.nostr_client(privkey).await?;
    log::debug!("Sending event: {event:?}");
    client.send_event(event).await?;

    Ok(())
}

fn new_event(create: &Create, privkey: &str) -> anyhow::Result<Event> {
    let keys = Keys::from_sk_str(privkey)?;
    let kind = NamespaceNostrKind::Name.into();
    let nsid = create.namespace_id()?.to_hex();
    let dtag = Tag::Generic(TagKind::D, vec![nsid]);
    let indtag = Tag::Generic("ind".into(), vec![create.name.clone()]);
    let tags = vec![dtag, indtag];
    let content = serde_json::to_string(&create.children)?;
    Ok(EventBuilder::new(kind, content, &tags).to_event(&keys)?)
}
