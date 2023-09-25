use std::collections::HashMap;

use anyhow::{anyhow, Context};
use itertools::Itertools;
use nostr_sdk::Event;
use secp256k1::XOnlyPublicKey;

use super::Nsid;

pub trait EventExtractor {
    fn extract_children(&self, name: &str) -> anyhow::Result<Vec<(String, XOnlyPublicKey)>>;
    fn extract_records(&self) -> anyhow::Result<HashMap<String, String>>;
    fn extract_name(&self) -> anyhow::Result<String>;
    fn extract_nsid(&self) -> anyhow::Result<Nsid>;
    fn extract_prev_nsid(&self) -> anyhow::Result<Option<Nsid>>;
}

impl EventExtractor for Event {
    fn extract_children(&self, name: &str) -> anyhow::Result<Vec<(String, XOnlyPublicKey)>> {
        let s: Vec<(String, XOnlyPublicKey)> =
            serde_json::from_str(&self.content).context("Invalid event content")?;
        let children = s
            .into_iter()
            .map(|(n, pk)| (format!("{n}.{name}"), pk))
            .collect_vec();
        Ok(children)
    }

    fn extract_records(&self) -> anyhow::Result<HashMap<String, String>> {
        Ok(serde_json::from_str(&self.content)?)
    }

    fn extract_name(&self) -> anyhow::Result<String> {
        let name = self
            .tags
            .iter()
            .filter_map(|t| match t {
                nostr_sdk::Tag::Generic(tk, values) => match tk {
                    nostr_sdk::prelude::TagKind::Custom(tn) if tn == "nom" => {
                        Some(values.iter().next()?.clone())
                    }
                    _ => None,
                },
                _ => None,
            })
            .next()
            .ok_or_else(|| anyhow!("Missing or invalid 'nom' tag"))?;
        Ok(name)
    }

    fn extract_nsid(&self) -> anyhow::Result<Nsid> {
        let nsid = self
            .tags
            .iter()
            .filter_map(|t| match t {
                nostr_sdk::Tag::Identifier(id) => Some(id.clone()),
                _ => None,
            })
            .next()
            .ok_or_else(|| anyhow!("Missing 'd' tag"))?;
        nsid.parse()
    }

    fn extract_prev_nsid(&self) -> anyhow::Result<Option<Nsid>> {
        let nn = self
            .tags
            .iter()
            .find_map(|t| match t {
                nostr_sdk::Tag::Generic(tk, values) => match tk {
                    nostr_sdk::prelude::TagKind::Custom(tn) if tn == "nom" => {
                        Some(values.get(1)?.clone())
                    }
                    _ => None,
                },
                _ => None,
            })
            .and_then(|s| s.parse::<Nsid>().ok());
        Ok(nn)
    }
}
