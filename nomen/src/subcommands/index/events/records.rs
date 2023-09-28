use std::time::Duration;

use nomen_core::util::NameKind;
use nostr_sdk::{Event, Filter};
use sqlx::SqlitePool;

use crate::{config::Config, db, subcommands::index::events::EventData};

pub async fn records(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    tracing::info!("Beginning indexing record events.");
    let events = latest_events(config, pool).await?;
    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => save_event(pool, ed).await?,
            Err(err) => tracing::debug!("Invalid event: {err}"),
        }
    }

    tracing::info!("Records events indexing complete.");
    Ok(())
}

async fn save_event(pool: &SqlitePool, ed: EventData) -> anyhow::Result<()> {
    tracing::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        fingerprint,
        nsid: _,
        calculated_nsid,
        pubkey,
        name,
        created_at,
        raw_content,
        records: _,
        raw_event,
    } = ed;
    db::insert_name_event(
        pool,
        name,
        fingerprint,
        calculated_nsid,
        pubkey,
        created_at,
        event_id,
        raw_content,
        raw_event,
    )
    .await?;

    Ok(())
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let index_height = db::last_records_time(pool).await?;
    let filter = Filter::new()
        .kind(NameKind::Name.into())
        .since(index_height.into());

    let (_keys, client) = config.nostr_random_client().await?;
    let events = client
        .get_events_of(vec![filter], Some(Duration::from_secs(10)))
        .await?;
    client.disconnect().await?;
    Ok(events)
}
