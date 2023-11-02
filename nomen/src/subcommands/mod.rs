mod index;
mod server;
pub mod util;

pub use index::*;
pub use server::*;
use sqlx::SqlitePool;

use crate::config::{Config, ConfigFile};

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
    sqlx::query("DELETE FROM blockchain_index WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM transfer_cache WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM old_transfer_cache WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(pool)
        .await?;
    Ok(())
}
