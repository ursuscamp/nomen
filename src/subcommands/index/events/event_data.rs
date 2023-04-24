use std::collections::HashMap;

use anyhow::bail;
use bitcoin::XOnlyPublicKey;
use nostr_sdk::{Event, EventId};

use crate::util::{EventExtractor, Nsid};

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_id: EventId,
    pub nsid: Nsid,
    pub calculated_nsid: Nsid,
    pub pubkey: XOnlyPublicKey,
    pub name: String,
    pub created_at: i64,
    pub raw_content: String,
    pub records: Option<HashMap<String, String>>,
}

impl EventData {
    pub fn from_event(event: &Event) -> anyhow::Result<Self> {
        let nsid = event.extract_nsid()?;
        let calculated_nsid = event.clone().try_into()?;
        let name = event.extract_name()?;
        let records = event.extract_records().ok();

        Ok(EventData {
            event_id: event.id,
            nsid,
            calculated_nsid,
            pubkey: event.pubkey,
            name,
            created_at: event.created_at.as_i64(),
            raw_content: event.content.clone(),
            records,
        })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.nsid != self.calculated_nsid {
            bail!("Invalid nsid")
        }
        // TODO: validate name regex here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_data() {
        let event = r#"{"id":"468dcefb10b9c5cef7451129beb7be37a266af063ac76c259a537822871b9d88","pubkey":"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f","created_at":1681352172,"kind":38300,"tags":[["d","28d63a9a61c6c5ce6be37a830105c92cf7a8f365"],["ind","smith"]],"content":"","sig":"b00a78ff3901063deb9b915d7cb17afa5a5fbe8be3cbf2808f3281dabd2ab134b4b32b380db88aa4c9677d1b870346f6594948cdd7abf5a7897ae3480347c6d8"}"#;
        let event = Event::from_json(event).unwrap();
        let mut ed: EventData = EventData::from_event(&event).unwrap();
        assert!(ed.validate().is_ok());

        ed.nsid = Nsid::from_slice(&[0; 20]).unwrap();
        assert!(ed.validate().is_err());
    }
}
