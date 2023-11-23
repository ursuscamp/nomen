use std::collections::HashMap;

use nostr_sdk::{EventBuilder, Keys, Tag};
use secp256k1::SecretKey;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db::{self, relay_index::Name},
};

pub async fn publish(config: &Config, pool: &SqlitePool, use_queue: bool) -> anyhow::Result<()> {
    if !config.publish_index() {
        return Ok(());
    }
    let sk: SecretKey = config
        .secret_key()
        .expect("Missing config validation for secret")
        .into();
    let keys = Keys::new(sk);
    let (_, client) = config.nostr_random_client().await?;

    tracing::info!("Publishing relay index.");
    let names = if use_queue {
        db::relay_index::fetch_all_queued(pool).await?
    } else {
        db::relay_index::fetch_all(pool).await?
    };
    send_events(pool, names, keys, &client).await?;
    tracing::info!("Publishing relay index complete.");

    client.disconnect().await.ok();
    Ok(())
}

async fn send_events(
    conn: &SqlitePool,
    names: Vec<Name>,
    keys: Keys,
    client: &nostr_sdk::Client,
) -> Result<(), anyhow::Error> {
    for name in names {
        let records: HashMap<String, String> = serde_json::from_str(&name.records)?;
        let content = Content {
            name: name.name.clone(),
            pubkey: name.pubkey,
            records,
        };
        let content_serialize = serde_json::to_string(&content)?;
        let event = EventBuilder::new(
            nostr_sdk::Kind::ParameterizedReplaceable(38301),
            content_serialize,
            &[Tag::Identifier(name.name.clone())],
        )
        .to_event(&keys)?;

        match client.send_event(event.clone()).await {
            Ok(s) => {
                tracing::info!("Broadcast event id {s}");
                db::relay_index::delete(conn, &name.name).await?;
            }
            Err(e) => {
                tracing::error!(
                    "Unable to broadcast event {} during relay index publish: {e}",
                    event.id
                );
            }
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct Content {
    name: String,
    pubkey: String,
    records: HashMap<String, String>,
}
