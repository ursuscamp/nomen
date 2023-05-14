use nostr_sdk::{Event, Filter};
use sqlx::SqlitePool;

use crate::{
    config::{Cli, Config},
    db,
    subcommands::index::events::EventData,
    util::NameKind,
};

pub async fn transfer(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning indexing transfer events.");
    let events = latest_events(config, pool).await?;
    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => save_event(pool, ed).await?,
            Err(err) => log::debug!("Invalid event: {err}"),
        }
    }

    log::info!("Records events transfer complete.");
    Ok(())
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let index_height = db::last_transfer_time(pool).await?;
    let filter = Filter::new()
        .kind(NameKind::Transfer.into())
        .since(index_height.into());

    let (_keys, client) = config.nostr_random_client().await?;
    Ok(client.get_events_of(vec![filter], None).await?)
}

async fn save_event(pool: &SqlitePool, ed: EventData) -> anyhow::Result<()> {
    log::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        fingerprint,
        nsid,
        calculated_nsid: _,
        pubkey,
        name,
        created_at,
        raw_content,
        records: _,
        raw_event,
    } = ed;

    db::insert_transfer_event(
        pool,
        nsid,
        pubkey,
        created_at,
        event_id,
        name,
        fingerprint,
        raw_content,
        raw_event,
    )
    .await?;

    Ok(())
}
