use crate::{config::Config, db};

mod blockchain;
pub mod events;

pub async fn index(config: &Config) -> anyhow::Result<()> {
    let pool = config.sqlite().await?;
    blockchain::index(config, &pool).await?;
    events::records(config, &pool).await?;
    events::relay_index::publish(config, &pool, true).await?;

    db::event_log::save(&pool, "index", "").await?;
    Ok(())
}
