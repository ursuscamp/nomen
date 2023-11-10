use crate::{config::Config, db};

mod blockchain;
mod events;

pub async fn index(config: &Config) -> anyhow::Result<()> {
    let pool = config.sqlite().await?;
    blockchain::index(config, &pool).await?;
    events::records(config, &pool).await?;
    events::relay_index::publish(config, &pool).await?;

    db::save_event(&pool, "index", "").await?;
    Ok(())
}
