use nostr_sdk::{Event, Filter};
use sqlx::SqlitePool;

use crate::{config::Config, db, subcommands::index::events::EventData, util::NameKind};

pub async fn records(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning indexing record events.");
    let events = latest_events(config, pool).await?;
    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => save_event(pool, ed).await?,
            Err(err) => log::debug!("Invalid event: {err}"),
        }
    }

    log::info!("Records events indexing complete.");
    Ok(())
}

async fn save_event(pool: &SqlitePool, ed: EventData) -> anyhow::Result<()> {
    log::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        nsid: _,
        pubkey,
        name,
        created_at,
        raw_content,
        records: _,
    } = ed;
    db::insert_records_event(pool, pubkey, created_at, event_id, name, raw_content).await?;

    Ok(())
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let index_height = db::last_records_time(pool).await?;
    let filter = Filter::new()
        .kind(NameKind::Record.into())
        .since(index_height.into());

    let (_keys, client) = config.nostr_random_client().await?;
    Ok(client.get_events_of(vec![filter], None).await?)
}
