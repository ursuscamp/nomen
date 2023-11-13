mod index;
mod server;
pub mod util;

pub use index::*;
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
    tracing::info!("Re-indexing blockchain from blockheight {blockheight}.");
    db::index::reindex(pool, blockheight).await?;
    Ok(())
}

pub(crate) async fn rescan(
    _config: &Config,
    pool: &SqlitePool,
    blockheight: i64,
) -> anyhow::Result<()> {
    tracing::info!("Re-scanning blockchain from blockheight {blockheight}.");
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
