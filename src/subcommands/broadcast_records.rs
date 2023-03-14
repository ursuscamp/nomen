use std::{path::Path, str::FromStr};

use anyhow::anyhow;
use nostr_sdk::{
    prelude::{FromSkStr, TagKind},
    EventBuilder, Keys, Tag,
};

use crate::{
    config::{Cli, Config},
    db,
    documents::{self, ExampleDocument},
    util::{NamespaceNostrKind, Nsid},
};

pub fn example_records() -> anyhow::Result<()> {
    let doc = serde_json::to_string_pretty(&documents::Records::create_example())?;
    println!("{doc}");

    Ok(())
}

pub async fn broadcast_records(
    config: &Config,
    document: &Path,
    privkey: &str,
) -> anyhow::Result<()> {
    let records: documents::Records = serde_json::from_str(&std::fs::read_to_string(document)?)?;
    let (keys, client) = config.nostr_client(privkey).await?;
    let kind = NamespaceNostrKind::Record.into();
    let nsid = nsid(&config, &records.name)
        .await?
        .ok_or(anyhow!("Namespace not found for name"))?;
    let dtag = Tag::Generic(TagKind::D, vec![nsid.to_string()]);
    let indtag = Tag::Generic("ind".into(), vec![records.name.clone()]);
    let tags = vec![dtag, indtag];
    let content = serde_json::to_string(&records.records)?;
    let event = EventBuilder::new(kind, content, &tags).to_event(&keys)?;

    client.send_event(event).await?;
    Ok(())
}

async fn nsid(config: &Config, name: &str) -> anyhow::Result<Option<Nsid>> {
    let conn = config.sqlite().await?;
    let index = db::nsid_for_name(&conn, name.to_owned())
        .await
        .map_err(|_| anyhow!("Query error. Perhaps missing name?"))?;
    index.map(|s| Nsid::from_str(&s)).transpose()
}
