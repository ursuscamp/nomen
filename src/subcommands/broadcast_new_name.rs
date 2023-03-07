use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::{connect, Message};

use crate::{config::Config, nostr::Event};

pub fn broadcast_new_name(
    config: &Config,
    namespace_id: &String,
    privkey: &String,
) -> anyhow::Result<()> {
    let event = Event::new_broadcast_name(&namespace_id).finalize(&privkey)?;
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
