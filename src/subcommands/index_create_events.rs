use std::{
    collections::{HashMap, VecDeque},
    net::TcpStream,
    time::Duration,
};

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use nostr_sdk::{Client, Event, Filter};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{config::Config, db, name::Namespace, util::NamespaceNostrKind, validators};

pub async fn index_create_events(config: &Config) -> anyhow::Result<()> {
    let (_keys, client) = config.nostr_random_client().await?;
    let mut conn = config.sqlite().await?;

    for event in search_relays(&conn, &client).await? {
        // Validate event parameters
        validators::event::create(&event)?;

        let ns: Namespace = match event.clone().try_into() {
            Ok(ns) => ns,
            Err(e) => {
                log::error!("Skipping invalid event (err: {e}): {event:?}");
                continue;
            }
        };
        db::insert_create_event(&conn, event, ns.clone()).await?;
        index_namespace_tree(&conn, &ns).await?;
    }

    Ok(())
}

async fn search_relays(conn: &SqlitePool, client: &Client) -> anyhow::Result<Vec<Event>> {
    log::debug!("Searching relays for new create events");
    let created_at = db::last_create_event_time(conn).await?;
    let filters = Filter::new()
        .kind(NamespaceNostrKind::Name.into())
        .since(created_at.into());
    Ok(client.get_events_of(vec![filters], None).await?)
}

async fn index_namespace_tree(conn: &SqlitePool, ns: &Namespace) -> anyhow::Result<()> {
    let root = ns.namespace_id();

    // Use a work queue to push names to process
    let mut queue = VecDeque::new();
    queue.push_back((None, String::new(), ns.clone()));

    while queue.len() > 0 {
        let (parent_nsid, parent_name, next) = queue.pop_front().unwrap(); // Queue already verified > 0 elements
        let nsid = next.namespace_id().to_hex();
        let fqdn = if parent_name.is_empty() {
            next.0.clone()
        } else {
            format!("{}.{}", next.0.clone(), parent_name)
        };
        queue.extend(
            next.2
                .into_iter()
                .map(|n| (Some(nsid.clone()), fqdn.clone(), n)),
        );
        db::index_name_nsid(
            conn,
            nsid,
            fqdn,
            root.to_hex(),
            parent_nsid,
            next.1.to_hex(),
        )
        .await?;
    }
    Ok(())
}
