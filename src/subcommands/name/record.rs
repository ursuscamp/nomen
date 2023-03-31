use std::collections::HashMap;

use nostr_sdk::{prelude::TagKind, EventBuilder, Tag};

use crate::{
    config::{Config, NameRecordSubcomand},
    subcommands::get_keys,
    util::NameKind,
};

pub async fn record(config: &Config, record_data: &NameRecordSubcomand) -> anyhow::Result<()> {
    let keys = get_keys(&record_data.privkey)?;
    let map: HashMap<String, String> = record_data
        .records
        .iter()
        .map(|p| p.clone().pair())
        .collect();
    let records = serde_json::to_string(&map)?;

    let event = EventBuilder::new(
        NameKind::Record.into(),
        records,
        &[
            Tag::Identifier(record_data.nsid.to_string()),
            Tag::Generic(
                TagKind::Custom("ind".to_owned()),
                vec![record_data.name.clone()],
            ),
        ],
    )
    .to_event(&keys)?;

    let (_keys, client) = config.nostr_random_client().await?;
    let event_id = client.send_event(event).await?;
    println!("Sent event {event_id}");

    Ok(())
}

fn parse_records(records: &[String]) -> HashMap<String, String> {
    records
        .iter()
        .filter_map(|rec| rec.split_once('='))
        .map(|(k, v)| (k.to_uppercase(), v.to_owned()))
        .collect()
}
