use sqlx::SqlitePool;

use crate::{
    config::{Cli, Config},
    db,
};

mod blockchain;
mod events;
mod owners;

pub async fn index(config: &Config) -> anyhow::Result<()> {
    let pool = config.sqlite().await?;
    blockchain::raw_index(config, &pool).await?;
    // events::records(config, &pool).await?;
    // owners::reindex(&pool).await?;

    db::save_event(&pool, "index", "").await?;
    Ok(())
}
