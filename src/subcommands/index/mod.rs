use sqlx::SqlitePool;

use crate::{config::Config, db};

mod blockchain;
mod events;

pub async fn index(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    blockchain::index(config, pool).await?;
    events::create(config, pool).await?;
    events::records(config, pool).await?;

    db::save_event(pool, "index", "").await?;
    Ok(())
}
