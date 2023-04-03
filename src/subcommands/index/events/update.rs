use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use nostr_sdk::{Event, Filter};
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    subcommands::index::events::EventData,
    util::{NameKind, NsidBuilder},
};

pub async fn update(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning indexing create events.");
    let events = latest_events(config, pool).await?;

    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => match ed.validate() {
                Ok(_) => {
                    save_names(pool, &ed).await?;
                    save_event(pool, &ed).await?;
                    process_updates(pool, &ed).await?
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

pub async fn process_updates(pool: &SqlitePool, ed: &EventData) -> anyhow::Result<()> {
    let old_parent = ed
        .prev
        .ok_or_else(|| anyhow!("Previous parent must be present on event"))?;
    log::info!("Processing updates...");

    log::debug!("Copying names from previous blockchain update.");
    db::copy_name_nsid(pool, old_parent, ed.nsid).await?;

    log::debug!("Marking old blockchain update as inactive");
    sqlx::query("UPDATE blockchain SET status = 'replaced' WHERE nsid = ?")
        .bind(old_parent.to_hex())
        .execute(pool)
        .await?;

    log::info!("Update processing complete.");
    Ok(())
}

async fn save_event(pool: &SqlitePool, ed: &EventData) -> anyhow::Result<()> {
    log::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        nsid,
        prev,
        pubkey,
        name,
        created_at,
        raw_content,
        children,
        records,
    } = ed;

    db::insert_update_event(
        pool,
        *nsid,
        prev.ok_or_else(|| anyhow!("Previous NSID is missing but required"))?,
        *pubkey,
        *created_at,
        *event_id,
        name.clone(),
        raw_content.clone(),
    )
    .await?;

    Ok(())
}

async fn save_names(pool: &SqlitePool, ed: &EventData) -> anyhow::Result<()> {
    // db::index_name_nsid(pool, ed.nsid, &ed.name, Some(ed.nsid), ed.pubkey).await?;
    let children = ed
        .children
        .as_ref()
        .ok_or_else(|| anyhow!("No children found"))?;
    for (name, pubkey) in children {
        let nsid = NsidBuilder::new(name, &ed.pubkey).finalize();
        db::index_name_nsid(pool, nsid, name, Some(ed.nsid), *pubkey, true).await?;
    }

    Ok(())
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let (_keys, client) = config.nostr_random_client().await?;
    let since = db::last_update_event_time(pool).await?;
    let filter = Filter::new()
        .kind(NameKind::Update.into())
        .since(since.into());
    let events = client.get_events_of(vec![filter], None).await?;
    Ok(events)
}
