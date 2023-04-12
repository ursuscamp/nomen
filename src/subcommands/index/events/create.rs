use anyhow::{anyhow, bail, Context};
use bitcoin::XOnlyPublicKey;
use itertools::Itertools;
use nostr_sdk::{Event, EventId, Filter};
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    util::{NameKind, Nsid, NsidBuilder},
};

use super::EventData;

pub async fn create(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning indexing create events.");
    let events = latest_events(config, pool).await?;

    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => match ed.validate() {
                Ok(_) => {
                    save_event(pool, ed).await?;
                }
                Err(e) => {
                    log::debug!("{ed:#?}");
                    log::error!("Invalid event {} with err: {e}", event.id)
                }
            },
            Err(err) => log::debug!("Event {} with err: {err}", event.id),
        }
    }

    log::info!("Create event indexing complete.");
    Ok(())
}

async fn save_event(pool: &SqlitePool, ed: EventData) -> anyhow::Result<()> {
    log::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        nsid,
        pubkey,
        name,
        created_at,
        raw_content,
        records,
    } = ed;

    db::insert_create_event(pool, nsid, pubkey, created_at, event_id, name, raw_content).await?;

    Ok(())
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let (_keys, client) = config.nostr_random_client().await?;
    let since = db::last_create_event_time(pool).await?;
    let filter = Filter::new()
        .kind(NameKind::Name.into())
        .since(since.into());
    let events = client.get_events_of(vec![filter], None).await?;
    Ok(events)
}
