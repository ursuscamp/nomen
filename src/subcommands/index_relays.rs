use std::{net::TcpStream, time::Duration};

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use nostr_sdk::{Client, Event, Filter};
use serde_json::json;

use crate::{config::Config, db, name::Namespace, util::NamespaceNostrKind};

pub async fn index_relays(config: &Config) -> anyhow::Result<()> {
    let (_keys, client) = config.nostr_random_client().await?;
    let conn = config.sqlite().await?;

    for nsid in db::discovered_nsids(&conn).await? {
        match search_relays(&client, &nsid).await {
            Ok(ns) => {
                log::debug!("Namespace found: {ns:?}");
                db::update_from_relay(&conn, &ns).await?;
                index_namespace_tree(&config, &ns, "").await?;
            }
            Err(e) => log::error!("{e}"),
        }
    }

    Ok(())
}

async fn search_relays(client: &Client, nsid: &str) -> anyhow::Result<Namespace> {
    log::debug!("Searching for events for nsid {nsid}");
    let filters = Filter::new()
        .kind(NamespaceNostrKind::Name.into())
        .replaceable_event(nsid);
    client
        .get_events_of(vec![filters], Some(Duration::from_secs(1)))
        .await?
        .into_iter()
        .map(|e| e.try_into())
        .filter(Result::is_ok)
        .collect::<anyhow::Result<Vec<Namespace>>>()?
        .first()
        .cloned()
        .ok_or(anyhow!("No name events found"))
}

#[async_recursion::async_recursion]
async fn index_namespace_tree(
    config: &Config,
    namespace: &Namespace,
    parent_name: &str,
) -> anyhow::Result<()> {
    let conn = config.sqlite().await?;
    let fqdn = if parent_name.is_empty() {
        namespace.0.clone()
    } else {
        format!("{}.{}", namespace.0, parent_name)
    };

    db::index_name(&conn, fqdn.clone(), namespace.namespace_id().to_string()).await;
    for child in &namespace.2 {
        index_namespace_tree(config, child, &fqdn).await;
    }

    Ok(())
}
