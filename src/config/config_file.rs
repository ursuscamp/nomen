use std::path::PathBuf;

use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfig {
    pub bind: Option<String>,
    pub without_explorer: Option<bool>,
    pub without_api: Option<bool>,
    pub without_indexer: Option<bool>,
    pub indexer_delay: Option<u64>,
}
impl ServerConfig {
    fn init() -> ServerConfig {
        ServerConfig {
            bind: Some("0.0.0.0:8080".into()),
            without_explorer: Some(false),
            without_api: Some(false),
            without_indexer: Some(false),
            indexer_delay: Some(30),
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
    fn init() -> RpcConfig {
        RpcConfig {
            cookie: Some(
                "path to bitcoin cookie file (higher priority than username/password)".into(),
            ),
            user: Some("username (lower priority than cookie file)".into()),
            password: Some("password".into()),
            host: Some("localhost".into()),
            port: Some(8441),
            network: Some(Network::Bitcoin),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NostrConfig {
    pub relays: Option<Vec<String>>,
}
impl NostrConfig {
    fn init() -> NostrConfig {
        NostrConfig {
            relays: Some(vec![]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ConfigFile {
    pub data: Option<PathBuf>,
    pub nostr: NostrConfig,
    pub server: ServerConfig,
    pub rpc: RpcConfig,
}

impl ConfigFile {
    pub fn init() -> ConfigFile {
        ConfigFile {
            data: Some("nomen.db".into()),
            nostr: NostrConfig::init(),
            server: ServerConfig::init(),
            rpc: RpcConfig::init(),
        }
    }
}
