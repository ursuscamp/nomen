use std::net::TcpStream;

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use serde_json::json;

use crate::{
    config::Config,
    db::{self, IndexStatus, Namespace},
    nostr::Event,
};

pub fn index_relays(config: &Config) -> anyhow::Result<()> {
    let nstree = db::namespaces()?;

    for item in nstree.into_iter() {
        // let mut relays = connect_relays(config)?;

        let (nsid, model) = item?;
        let nsidh = nsid.to_hex();
        let ns = Namespace::decode(&model)?;

        if ns.status == IndexStatus::Detected {
            // let event = search_relays(&mut relays, &nsidh)?;
        }
    }

    Ok(())
}
