use std::net::TcpStream;

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use serde_json::json;
use tokio_tungstenite::tungstenite::{connect, Message, WebSocket};

use crate::{
    config::Config,
    db::{self, IndexStatus, Namespace},
    nostr::Event,
};

pub fn index_relays(config: &Config) -> anyhow::Result<()> {
    let nstree = db::namespaces()?;

    for item in nstree.into_iter() {
        let mut relays = connect_relays(config)?;

        let (nsid, model) = item?;
        let nsidh = nsid.to_hex();
        let ns = Namespace::decode(&model)?;

        if ns.status == IndexStatus::Detected {
            let event = search_relays(&mut relays, &nsidh)?;
        }
    }

    Ok(())
}

type WS = WebSocket<tokio_tungstenite::tungstenite::stream::MaybeTlsStream<TcpStream>>;

fn connect_relays(config: &Config) -> anyhow::Result<Vec<WS>> {
    let relays = match &config.relay {
        None => return Err(anyhow!("No relays configured")),
        Some(v) if v.is_empty() => return Err(anyhow!("No relays configured")),
        Some(relays) => relays,
    };
    let sockets = relays
        .into_iter()
        .map(|relay| connect(relay))
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .map(|r| r.0)
        .collect::<Vec<_>>();
    Ok(sockets)
}

fn search_relays(relays: &mut [WS], nsid: &str) -> anyhow::Result<Option<Event>> {
    let mut event = None;

    for mut relay in relays {
        let filter = json!(["REQ", nsid, {"kinds": [38300], "#d": [nsid]}]).to_string();
        relay.write_message(Message::text(filter))?;
        let recv = relay.read_message()?;
        println!("Message: {recv}");
        // let name: Name = recv.into_text()?
        relay.write_message(Message::text(json!(["CLOSE", nsid]).to_string()))?;
    }

    Ok(event)
}
