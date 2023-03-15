use std::str::FromStr;

use nostr_sdk::{
    prelude::{tag, TagKind},
    Event, Kind,
};

use super::{NamespaceNostrKind, Nsid};

pub trait MetadataExtractor {
    fn extract_nsid(&self) -> Option<Nsid>;
    fn extract_name(&self) -> Option<String>;
}

impl MetadataExtractor for Event {
    fn extract_nsid(&self) -> Option<Nsid> {
        self.tags
            .iter()
            .filter_map(|tag| match tag {
                nostr_sdk::Tag::Identifier(tag) => Some(Nsid::from_str(&tag)),
                _ => None,
            })
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .nth(0)
    }

    fn extract_name(&self) -> Option<String> {
        self.tags
            .iter()
            .filter_map(|tag| match tag {
                nostr_sdk::Tag::Generic(TagKind::Custom(tk), tag) if tk == "ind" => {
                    tag.first().cloned()
                }
                _ => None,
            })
            .nth(0)
    }
}
