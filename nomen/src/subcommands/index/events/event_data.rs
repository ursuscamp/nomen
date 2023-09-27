use std::collections::HashMap;

use anyhow::bail;
use nostr_sdk::{Event, EventId};
use secp256k1::XOnlyPublicKey;

use crate::util::{EventExtractor, Hash160, Name, Nsid};

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_id: EventId,
    pub fingerprint: [u8; 5],
    pub nsid: Nsid,
    pub calculated_nsid: Nsid,
    pub pubkey: XOnlyPublicKey,
    pub name: Name,
    pub created_at: i64,
    pub raw_content: String,
    pub records: Option<HashMap<String, String>>,
    pub raw_event: String,
}

impl EventData {
    pub fn from_event(event: &Event) -> anyhow::Result<Self> {
        let nsid = event.extract_nsid()?;
        let calculated_nsid = event.clone().try_into()?;
        let name = event.extract_name()?;
        let fingerprint = Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint();
        let records = event.extract_records().ok();
        let raw_event = serde_json::to_string(event)?;

        Ok(EventData {
            event_id: event.id,
            fingerprint,
            nsid,
            calculated_nsid,
            pubkey: event.pubkey,
            name: name.parse()?,
            created_at: event.created_at.as_i64(),
            raw_content: event.content.clone(),
            records,
            raw_event,
        })
    }

    #[allow(unused)]
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.nsid != self.calculated_nsid {
            bail!("Invalid nsid")
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_data() {
        let event = r#"{"id":"4fb5485ad12706f3ddbde1cdeab3199fcbef01b4c2456a7420ef5acb400d29e5","pubkey":"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f","created_at":1682476154,"kind":38300,"tags":[["d","28d63a9a61c6c5ce6be37a830105c92cf7a8f365"],["nom","smith"]],"content":"{\"IP4\":\"127.0.0.1\",\"NPUB\":\"npub1234\"}","sig":"53a629c8169c29abc971653b71ebf8ceb185735170b702dd48377a3336819680577ef28a257b8e4db5e8101531232e1c886a35721b5af1399c32cb526fd61bb6"}"#;
        let event = Event::from_json(event).unwrap();
        let mut ed: EventData = EventData::from_event(&event).unwrap();
        assert!(ed.validate().is_ok());

        ed.nsid = Nsid::from_slice(&[0; 20]).unwrap();
        assert!(ed.validate().is_err());
    }
}
