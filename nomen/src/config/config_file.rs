use std::path::PathBuf;

use bitcoin::Network;
use nostr_sdk::Keys;
use serde::{Deserialize, Serialize};

use crate::util::Nsec;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfig {
    pub bind: Option<String>,
    pub explorer: Option<bool>,
    pub api: Option<bool>,
    pub indexer: Option<bool>,
    pub indexer_delay: Option<u64>,
    pub confirmations: Option<usize>,
}
impl ServerConfig {
    fn example() -> ServerConfig {
        ServerConfig {
            bind: Some("0.0.0.0:8080".into()),
            explorer: Some(true),
            api: Some(true),
            indexer: Some(true),
            indexer_delay: Some(30),
            confirmations: Some(3),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RpcConfig {
    pub cookie: Option<PathBuf>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub network: Option<Network>,
}
impl RpcConfig {
    fn example() -> RpcConfig {
        RpcConfig {
            cookie: Some("path/to/cookie/file".into()),
            user: Some("rpc username".into()),
            password: Some("rpc password".into()),
            host: Some("localhost".into()),
            port: Some(8441),
            network: Some(Network::Bitcoin),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NostrConfig {
    pub relays: Option<Vec<String>>,
    pub secret: Option<Nsec>,
    pub publish: bool,
    pub well_known: bool,
}
impl NostrConfig {
    fn example() -> NostrConfig {
        NostrConfig {
            relays: Some(vec!["wss://relay.damus.io".into()]),
            secret: Keys::generate()
                .secret_key()
                .ok()
                .map(std::convert::Into::into),
            publish: true,
            well_known: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigFile {
    pub data: Option<PathBuf>,
    pub nostr: NostrConfig,
    pub server: ServerConfig,
    pub rpc: RpcConfig,
}

impl ConfigFile {
    pub fn example() -> ConfigFile {
        ConfigFile {
            data: Some("nomen.db".into()),
            nostr: NostrConfig::example(),
            server: ServerConfig::example(),
            rpc: RpcConfig::example(),
        }
    }
}
