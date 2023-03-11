use std::{net::TcpStream, time::Duration};

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use nostr_sdk::{Client, Event, Filter};
use serde_json::json;
use sled::{
    transaction::{ConflictableTransactionError, TransactionError},
    Transactional,
};

use crate::{
    config::Config,
    db::{self, names_nsid, IndexStatus, NamespaceModel},
    name::Namespace,
    util::NamespaceNostrKind,
};

pub async fn index_relays(config: &Config) -> anyhow::Result<()> {
    let nstree = db::namespaces()?;
    let (_keys, client) = config.nostr_random_client().await?;

    for item in nstree.into_iter() {
        let (nsid, model) = item?;
        let nsidh = nsid.to_hex();
        let mut model = NamespaceModel::decode(&model)?;

        if model.status == IndexStatus::Detected {
            match search_relays(&client, &nsidh).await {
                Ok(ns) => {
                    log::debug!("Namespace found: {ns:?}");
                    let mut model = model.clone();
                    model.status = IndexStatus::Indexed;
                    let encmodel = model.encode()?;
                    let names_nsid = names_nsid()?;
                    names_nsid.insert(&ns.0, ns.namespace_id().as_ref())?;
                    nstree.insert(model.nsid, encmodel);
                }
                Err(e) => log::error!("{e}"),
            }
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
