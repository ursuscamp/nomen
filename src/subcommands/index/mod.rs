use sqlx::SqlitePool;

use crate::config::Config;

mod blockchain;

pub async fn index(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    blockchain::index(config, pool).await?;
    Ok(())
}
