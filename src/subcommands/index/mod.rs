use sqlx::SqlitePool;

use crate::config::Config;

mod blockchain;
mod events;

pub async fn index(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    blockchain::index(config, pool).await?;
    events::create(config, pool).await?;
    events::records(config, pool).await?;
    events::update(config, pool).await?;
    Ok(())
}
