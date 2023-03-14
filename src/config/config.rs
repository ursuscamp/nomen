use std::{
    ops::Sub,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, Network};
use bitcoincore_rpc::RpcApi;
use clap::Parser;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Options, Relay,
};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;

use super::{Cli, ConfigFile, Subcommand};

#[derive(Debug, Clone)]
pub struct Config {
    cli: Cli,
    config_file: ConfigFile,
}

impl Config {
    pub fn new(cli: Cli, config_file: ConfigFile) -> Config {
        Config { cli, config_file }
    }

    pub fn subcommand(&self) -> &Subcommand {
        &self.cli.subcommand
    }

    pub fn data(&self) -> PathBuf {
        self.cli
            .data
            .as_ref()
            .or(self.config_file.data.as_ref())
            .cloned()
            .unwrap_or_else(|| PathBuf::from_str("indigo.db").unwrap())
    }

    pub fn rpchost(&self) -> Option<String> {
        self.cli
            .rpchost
            .as_ref()
            .or(self.config_file.rpchost.as_ref())
            .cloned()
    }

    pub fn rpcport(&self) -> Option<u16> {
        self.cli.rpcport.or(self.config_file.rpcport)
    }

    pub fn rpcuser(&self) -> Option<String> {
        self.cli
            .rpcuser
            .as_ref()
            .or(self.config_file.rpcuser.as_ref())
            .cloned()
    }

    pub fn rpcpass(&self) -> Option<String> {
        self.cli
            .rpcpass
            .as_ref()
            .or(self.config_file.rpcpass.as_ref())
            .cloned()
    }

    pub fn cookie(&self) -> Option<PathBuf> {
        self.cli
            .cookie
            .as_ref()
            .or(self.config_file.cookie.as_ref())
            .cloned()
    }

    pub fn relays(&self) -> Vec<String> {
        self.cli
            .relays
            .as_ref()
            .or_else(|| self.config_file.relays.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                vec![
                    "wss://relay.damus.io".into(),
                    "wss://relay.snort.social".into(),
                ]
            })
    }

    pub fn network(&self) -> Network {
        self.cli
            .network
            .or(self.config_file.network)
            .unwrap_or(Network::Bitcoin)
    }

    pub fn rpc_client(&self) -> anyhow::Result<bitcoincore_rpc::Client> {
        let host = self.rpchost();
        let port = self.rpcport();
        let user = self.rpcuser();
        let pass = self.rpcpass();
        let url = if host.is_some() && port.is_some() {
            format!("{}.{}", host.unwrap(), port.unwrap())
        } else {
            String::new()
        };
        let auth = if let Some(cookie) = self.cookie() {
            bitcoincore_rpc::Auth::CookieFile(cookie.clone())
        } else if user.is_some() && pass.is_some() {
            bitcoincore_rpc::Auth::UserPass(user.unwrap(), pass.unwrap())
        } else {
            bitcoincore_rpc::Auth::None
        };
        Ok(bitcoincore_rpc::Client::new(&url, auth)?)
    }

    pub async fn sqlite(&self) -> anyhow::Result<tokio_rusqlite::Connection> {
        Ok(Connection::open(&self.data()).await?)
    }

    pub async fn nostr_client(
        &self,
        sk: &str,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::from_sk_str(sk)?;
        let mut client =
            nostr_sdk::Client::new_with_opts(&keys, Options::new().wait_for_send(true));
        let relays = self.relays();
        for relay in relays {
            client.add_relay(relay, None).await?;
        }
        client.connect().await;
        Ok((keys, client))
    }

    pub async fn nostr_random_client(
        &self,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::generate();
        let sk = keys.secret_key()?.to_bech32()?;
        self.nostr_client(&sk).await
    }
}
