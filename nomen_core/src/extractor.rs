use std::collections::HashMap;

use itertools::Itertools;
use nostr_sdk::Event;
use secp256k1::XOnlyPublicKey;

use super::Nsid;

#[derive(thiserror::Error, Debug)]
#[error("event extractor")]
pub struct ExtractorError;

pub trait EventExtractor {
    fn extract_children(&self, name: &str)
        -> Result<Vec<(String, XOnlyPublicKey)>, ExtractorError>;
    fn extract_records(&self) -> Result<HashMap<String, String>, ExtractorError>;
    fn extract_name(&self) -> Result<String, ExtractorError>;
    fn extract_nsid(&self) -> Result<Nsid, ExtractorError>;
    fn extract_prev_nsid(&self) -> Result<Option<Nsid>, ExtractorError>;
}

impl EventExtractor for Event {
    fn extract_children(
        &self,
        name: &str,
    ) -> Result<Vec<(String, XOnlyPublicKey)>, ExtractorError> {
        let s: Vec<(String, XOnlyPublicKey)> =
            serde_json::from_str(&self.content).or(Err(ExtractorError))?;
        let children = s
            .into_iter()
            .map(|(n, pk)| (format!("{n}.{name}"), pk))
            .collect_vec();
        Ok(children)
    }

    fn extract_records(&self) -> Result<HashMap<String, String>, ExtractorError> {
        serde_json::from_str(&self.content).or(Err(ExtractorError))
    }

    fn extract_name(&self) -> Result<String, ExtractorError> {
        self.tags
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
            .ok_or(ExtractorError)
    }

    fn extract_nsid(&self) -> Result<Nsid, ExtractorError> {
        self.tags
            .iter()
            .filter_map(|t| match t {
                nostr_sdk::Tag::Identifier(id) => Some(id.clone()),
                _ => None,
            })
            .next()
            .ok_or(ExtractorError)?
            .parse()
            .or(Err(ExtractorError))
    }

    fn extract_prev_nsid(&self) -> Result<Option<Nsid>, ExtractorError> {
        let nsid = self
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
        Ok(nsid)
    }
}
