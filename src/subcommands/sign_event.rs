use nostr_sdk::UnsignedEvent;

use crate::config::{Config, SignEventCommand};

use super::get_keys;

pub async fn sign_event(config: &Config, args: &SignEventCommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let event: UnsignedEvent = serde_json::from_str(&args.event)?;
    let event = event.sign(&keys)?;

    if args.broadcast {
        let (_k, nostr) = config.nostr_random_client().await?;
        let event_id = nostr.send_event(event).await?;
        println!("Broadcast event {event_id}");
    } else {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}
