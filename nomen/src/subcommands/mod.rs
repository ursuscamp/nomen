mod index;
mod server;
pub mod util;

pub use index::*;
use nostr_sdk::Event;
pub use server::*;
use sqlx::SqlitePool;

use crate::{
    config::{Config, ConfigFile},
    db,
};

pub(crate) fn init() -> anyhow::Result<()> {
    let config_file = ConfigFile::example();
    let cfg = toml::to_string(&config_file)?;
    println!("{cfg} ");
    Ok(())
}

pub(crate) async fn reindex(
    _config: &Config,
    pool: &SqlitePool,
    blockheight: i64,
) -> anyhow::Result<()> {
    println!("Re-indexing blockchain from blockheight {blockheight}.");
    db::index::reindex(pool, blockheight).await?;
    Ok(())
}

pub(crate) async fn rescan(
    _config: &Config,
    pool: &SqlitePool,
    blockheight: i64,
) -> anyhow::Result<()> {
    println!("Re-scanning blockchain from blockheight {blockheight}.");
    db::index::reindex(pool, blockheight).await?;
    sqlx::query("DELETE FROM index_height WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM raw_blockchain WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) fn version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("Current version is {version}");
}

pub(crate) async fn rebroadcast(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    let events = sqlx::query_as::<_, (String,)>(
        "select ne.raw_event from valid_names_vw vn join name_events ne on vn.nsid = ne.nsid;",
    )
    .fetch_all(pool)
    .await?;
    println!(
        "Rebroadcasing {} events to {} relays",
        events.len(),
        config.relays().len()
    );
    let (_, client) = config.nostr_random_client().await?;
    for (event,) in events {
        let event = Event::from_json(event)?;
        client.send_event(event).await?;
    }

    Ok(())
}

pub(crate) async fn publish(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("Publishing full relay index");
    index::events::relay_index::publish(config, pool).await
}
