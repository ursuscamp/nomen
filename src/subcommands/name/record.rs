use std::{collections::HashMap, sync::Arc};

use nostr_sdk::{prelude::TagKind, EventBuilder, Tag};

use crate::{
    config::{Config, NameRecordSubcomand},
    subcommands::get_keys,
    util::{NameKind, NsidBuilder},
};

pub async fn record(config: &Config, record_data: &NameRecordSubcomand) -> anyhow::Result<()> {
    let keys = get_keys(&record_data.privkey)?;
    let nsid = NsidBuilder::new(&record_data.name, &keys.public_key()).finalize();
    let map: HashMap<String, String> = record_data
        .records
        .iter()
        .map(|p| p.clone().pair())
        .collect();
    let records = serde_json::to_string(&map)?;

    let event = super::name_event(keys.public_key(), &map, &record_data.name)?.sign(&keys)?;

    let (_keys, client) = config.nostr_random_client().await?;
    let event_id = client.send_event(event).await?;
    println!("Sent event {event_id}");

    Ok(())
}
