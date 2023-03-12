use std::path::Path;

use anyhow::anyhow;
use nostr_sdk::{
    prelude::{FromSkStr, TagKind},
    EventBuilder, Keys, Tag,
};

use crate::{
    config::Config,
    db::names_nsid,
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
    let nsid = nsid(&records.name)?.ok_or(anyhow!("Namespace not found for name"))?;
    let dtag = Tag::Generic(TagKind::D, vec![nsid.to_string()]);
    let indtag = Tag::Generic("ind".into(), vec![records.name.clone()]);
    let tags = vec![dtag, indtag];
    let content = serde_json::to_string(&records.records)?;
    let event = EventBuilder::new(kind, content, &tags).to_event(&keys)?;

    client.send_event(event).await?;
    Ok(())
}

fn nsid(name: &str) -> anyhow::Result<Option<Nsid>> {
    let index = names_nsid()?;
    index.get(&name)?.map(|n| Nsid::from_slice(&n)).transpose()
}
