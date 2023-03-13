use bitcoin::hashes::hex::ToHex;
use nostr_sdk::Filter;

use crate::{
    config::Config,
    db,
    util::{MetadataExtractor, NamespaceNostrKind},
};

pub async fn index_records_events(config: &Config) -> anyhow::Result<()> {
    let conn = config.sqlite().await?;
    let last_records_time = db::last_records_time(&conn).await?;
    log::debug!("Getting all record events sicne {last_records_time}");
    let filters = filters(last_records_time);
    let (_keys, client) = config.nostr_random_client().await?;
    let events = client.get_events_of(filters, None).await?;

    for event in events {
        let (nsid, pubkey, created_at, event_id, name) = match extract_record_data(&event) {
            Some(value) => value,
            None => continue,
        };
        log::debug!("Recording record for event {event:?}");

        db::insert_records_event(
            &conn,
            nsid,
            pubkey,
            created_at,
            event_id,
            name,
            event.content.clone(),
        )
        .await?;
    }

    Ok(())
}

fn filters(last_records_time: u64) -> Vec<Filter> {
    let filter = Filter::new()
        .kind(NamespaceNostrKind::Record.into())
        .since(last_records_time.into());
    let filters = vec![filter];
    filters
}

fn extract_record_data(event: &nostr_sdk::Event) -> Option<(String, String, u64, String, String)> {
    let nsid = match event.extract_nsid() {
        Some(nsid) => nsid.to_hex(),
        None => return None,
    };
    let pubkey = event.pubkey.to_hex();
    let created_at = event.created_at.as_u64();
    let event_id = event.id.to_hex();
    let name = match event.extract_name() {
        Some(name) => name,
        None => return None,
    };
    Some((nsid, pubkey, created_at, event_id, name))
}
