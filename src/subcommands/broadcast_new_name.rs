use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::{connect, Message};

use crate::{config::Config, documents::Create, nostr::Event};

pub fn broadcast_new_name(
    config: &Config,
    document: &Path,
    privkey: &String,
) -> anyhow::Result<()> {
    let create: Create = serde_json::from_str(&std::fs::read_to_string(document)?)?;
    let event =
        Event::new_broadcast_name(&create.namespace_id(), &create.name).finalize(&privkey)?;
    println!("{event:#?}");
    // let event = serde_json::to_string(&event)?;
    let envelope = json!(["EVENT", event]).to_string();
    println!("{envelope}");

    let relays = config.relay.as_ref().unwrap();
    for relay in relays {
        println!("Pushing event to relay {relay}");
        let (mut socket, _) = connect(relay)?;
        socket.write_message(Message::Text(envelope.clone()))?;
        let message = socket.read_message()?;
        println!("{message:#?}");
    }

    Ok(())
}
