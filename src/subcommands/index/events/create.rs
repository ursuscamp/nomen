use anyhow::{anyhow, bail, Context};
use bitcoin::XOnlyPublicKey;
use itertools::Itertools;
use nostr_sdk::{Event, EventId, Filter};
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    util::{NamespaceNostrKind, Nsid, NsidBuilder},
};

pub async fn create(config: &Config, pool: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning indexing create events.");
    let events = latest_events(config, pool).await?;

    for event in events {
        match EventData::from_event(&event) {
            Ok(ed) => match ed.validate() {
                Ok(_) => save_event(pool, ed).await?,
                Err(e) => log::error!("Invalid event {} with err: {e}", event.id),
            },
            Err(err) => log::debug!("Event {} with err: {err}", event.id),
        }
    }

    log::info!("Create event indexing complete.");
    Ok(())
}

async fn save_event(pool: &SqlitePool, ed: EventData) -> anyhow::Result<()> {
    log::info!("Saving valid event {}", ed.event_id);
    let EventData {
        event_id,
        nsid,
        pubkey,
        name,
        created_at,
        raw_children,
        children,
    } = ed;

    db::insert_create_event(pool, nsid, pubkey, created_at, event_id, name, raw_children).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct EventData {
    event_id: EventId,
    nsid: Nsid,
    pubkey: XOnlyPublicKey,
    name: String,
    created_at: i64,
    raw_children: String,
    children: Vec<(String, XOnlyPublicKey)>,
}

impl EventData {
    fn from_event(event: &Event) -> anyhow::Result<Self> {
        let nsid = extract_nsid(event)?.parse()?;
        let name = extract_name(event)?;
        let children = extract_children(event, &name)?;

        Ok(EventData {
            event_id: event.id,
            nsid,
            pubkey: event.pubkey,
            name,
            created_at: event.created_at.as_i64(),
            raw_children: event.content.clone(),
            children,
        })
    }

    fn validate(&self) -> anyhow::Result<()> {
        let mut builder = NsidBuilder::new(&self.name, &self.pubkey);
        for (name, pk) in &self.children {
            builder = builder.update_child(name, *pk);
        }
        let nsid = builder.finalize();
        if nsid != self.nsid {
            bail!("Invalid nsid")
        }
        Ok(())
    }
}

fn extract_children(event: &Event, name: &str) -> anyhow::Result<Vec<(String, XOnlyPublicKey)>> {
    let s: Vec<(String, XOnlyPublicKey)> =
        serde_json::from_str(&event.content).context("Invalid event content")?;
    let children = s
        .into_iter()
        .map(|(n, pk)| (format!("{n}.{name}"), pk))
        .collect_vec();
    Ok(children)
}

fn extract_name(event: &Event) -> anyhow::Result<String> {
    let name = event
        .tags
        .iter()
        .filter_map(|t| match t {
            nostr_sdk::Tag::Generic(tk, values) => match tk {
                nostr_sdk::prelude::TagKind::Custom(tn) if tn == "ind" => {
                    Some(values.iter().next()?.clone())
                }
                _ => None,
            },
            _ => None,
        })
        .next()
        .ok_or_else(|| anyhow!("Missing or invalid 'ind' tag"))?;
    Ok(name)
}

fn extract_nsid(event: &Event) -> anyhow::Result<String> {
    let nsid = event
        .tags
        .iter()
        .filter_map(|t| match t {
            nostr_sdk::Tag::Identifier(id) => Some(id.clone()),
            _ => None,
        })
        .next()
        .ok_or_else(|| anyhow!("Missing 'd' tag"))?;
    Ok(nsid)
}

async fn latest_events(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<Vec<Event>> {
    let (_keys, client) = config.nostr_random_client().await?;
    let since = db::last_create_event_time(pool).await?;
    let filter = Filter::new()
        .kind(NamespaceNostrKind::Name.into())
        .since(since.into());
    let events = client.get_events_of(vec![filter], None).await?;
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_data() {
        let event = r#"{"id":"d199153706fb15c0c055c443a2a95faa987ea3b35c8fc81dadc2d607f6fc7be4","pubkey":"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f","created_at":1679754877,"kind":38300,"tags":[["d","4e815dbf9d217f51ccbdfe3f24ac62a08ef8fed0"],["ind","smith"]],"content":"[[\"bob\",\"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f\"],[\"alice\",\"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f\"]]","sig":"1108abaf30ec221bf217e01463642912a8964fa536ad921e12ba3a7085ac57d135adbd6263a6256fc504eda8cc90b1b3d53c9fb74fb2078394b3cc29962785d0"}"#;
        let event = Event::from_json(event).unwrap();
        let mut ed: EventData = EventData::from_event(&event).unwrap();
        assert!(ed.validate().is_ok());

        ed.nsid = Nsid::from_slice(&[0; 20]).unwrap();
        assert!(ed.validate().is_err());
    }
}
