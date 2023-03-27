use std::collections::HashMap;

use anyhow::{anyhow, bail};
use bitcoin::XOnlyPublicKey;
use nostr_sdk::{Event, EventId};

use crate::util::{EventExtractor, Nsid, NsidBuilder};

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_id: EventId,
    pub nsid: Nsid,
    pub pubkey: XOnlyPublicKey,
    pub name: String,
    pub created_at: i64,
    pub raw_content: String,
    pub children: Option<Vec<(String, XOnlyPublicKey)>>,
    pub records: Option<HashMap<String, String>>,
}

impl EventData {
    pub fn from_event(event: &Event) -> anyhow::Result<Self> {
        let nsid = event.extract_nsid()?;
        let name = event.extract_name()?;
        let children = event.extract_children(&name).ok();
        let records = event.extract_records().ok();

        Ok(EventData {
            event_id: event.id,
            nsid,
            pubkey: event.pubkey,
            name,
            created_at: event.created_at.as_i64(),
            raw_content: event.content.clone(),
            children,
            records,
        })
    }

    pub fn validate_create(&self) -> anyhow::Result<()> {
        let nsid = self.recalc_nsid();
        if nsid != self.nsid {
            bail!("Invalid nsid")
        }
        Ok(())
    }

    pub fn recalc_nsid(&self) -> Nsid {
        let mut builder = NsidBuilder::new(&self.name, &self.pubkey);
        if let Some(children) = &self.children {
            for (n, pk) in children {
                builder = builder.update_child(n, *pk);
            }
        }
        builder.finalize()
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;

    use super::*;

    #[test]
    fn test_event_data() {
        let event = r#"{"id":"d199153706fb15c0c055c443a2a95faa987ea3b35c8fc81dadc2d607f6fc7be4","pubkey":"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f","created_at":1679754877,"kind":38300,"tags":[["d","4e815dbf9d217f51ccbdfe3f24ac62a08ef8fed0"],["ind","smith"]],"content":"[[\"bob\",\"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f\"],[\"alice\",\"d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f\"]]","sig":"1108abaf30ec221bf217e01463642912a8964fa536ad921e12ba3a7085ac57d135adbd6263a6256fc504eda8cc90b1b3d53c9fb74fb2078394b3cc29962785d0"}"#;
        let event = Event::from_json(event).unwrap();
        let mut ed: EventData = EventData::from_event(&event).unwrap();
        assert!(ed.validate_create().is_ok());

        ed.nsid = Nsid::from_slice(&[0; 20]).unwrap();
        assert!(ed.validate_create().is_err());
    }
}
