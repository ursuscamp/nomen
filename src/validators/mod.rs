pub mod event {
    use anyhow::anyhow;
    use nostr_sdk::Event;

    use crate::util::NamespaceNostrKind;

    pub fn create(event: &Event) -> anyhow::Result<bool> {
        if event.kind != NamespaceNostrKind::Name.into() {
            return Err(anyhow!("Incorrect kind"));
        }
        let valid = super::name::event(event)?;
        Ok(valid)
    }

    pub fn records(event: &Event) -> anyhow::Result<bool> {
        if event.kind != NamespaceNostrKind::Record.into() {
            return Err(anyhow!("Incorrect kind"));
        }
        let valid = super::name::event(event)?;
        Ok(valid)
    }
}

pub mod name {
    use anyhow::anyhow;
    use nostr_sdk::Event;
    use regex::Regex;

    use crate::util::MetadataExtractor;

    pub fn event(event: &Event) -> anyhow::Result<bool> {
        let ind_tag = event
            .extract_name()
            .ok_or_else(|| anyhow!("Event has no ind tag"))?;

        let r = Regex::new(r#"\A[a-z0-9\-]{3,256}\z"#).expect("Regex should parse");
        Ok(ind_tag.split('.').all(|s| r.is_match(s)))
    }
}

#[cfg(test)]
mod tests {
    use nostr_sdk::{prelude::TagKind, Event, EventBuilder, Keys, Tag};

    use crate::util::NamespaceNostrKind;

    use super::*;

    #[test]
    fn test_name_event() {
        let event = event(Some("hello-there.world"));
        assert!(name::event(&event).unwrap())
    }

    #[test]
    fn test_invalid_name_event() {
        let event = event(Some("no!way"));
        assert!(!name::event(&event).unwrap())
    }

    #[test]
    fn test_no_name_error() {
        let event = event(None);
        assert!(name::event(&event).is_err())
    }

    fn event(name: Option<&str>) -> Event {
        let tags = name
            .map(|s| vec![Tag::Generic(TagKind::Custom("ind".into()), vec![s.into()])])
            .unwrap_or_default();
        EventBuilder::new(NamespaceNostrKind::Name.into(), "[]", &tags)
            .to_event(&Keys::generate())
            .unwrap()
    }
}
