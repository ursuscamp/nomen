use std::{
    ops::Sub,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, Network};
use bitcoincore_rpc::RpcApi;
use clap::Parser;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Options,
};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct Config {
    /// Location of config file
    #[arg(short, long, default_value = ".indigo.toml")]
    #[serde(skip)]
    pub config: Option<PathBuf>,

    /// Path for index data.
    #[arg(short, long, default_value = ".indigo-data")]
    pub data: Option<PathBuf>,

    /// Location to Bitcoin Core cookie file.
    #[arg(long)]
    pub cookie: Option<PathBuf>,

    /// RPC username.
    #[arg(long)]
    pub rpcuser: Option<String>,

    /// RPC password.
    #[arg(long)]
    pub rpcpass: Option<String>,

    /// RPC host
    #[arg(long)]
    pub rpchost: Option<String>,

    /// RPC port number
    #[arg(long)]
    pub rpcport: Option<u16>,

    /// Bitcoin network
    #[arg(long)]
    pub network: Option<Network>,

    /// Nostr relays for commands that interact with relays.
    /// Can be specified multiple times.
    #[arg(long, short, action = clap::ArgAction::Append)]
    pub relay: Option<Vec<String>>,

    #[serde(skip)]
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

impl Config {
    pub fn merge_config_file(&self, config_file: &Config) -> Config {
        Config {
            config: self.config.clone(),
            data: self.data.clone(),
            cookie: self.cookie.clone().or(config_file.cookie.clone()),
            rpcuser: self.rpcuser.clone().or(config_file.rpcuser.clone()),
            rpcpass: self.rpcpass.clone().or(config_file.rpcpass.clone()),
            rpchost: self
                .rpchost
                .clone()
                .or(config_file.rpchost.clone())
                .or(Some("localhost".into())),
            rpcport: self
                .rpcport
                .clone()
                .or(config_file.rpcport.clone())
                .or(Some(8332)),
            network: config_file
                .network
                .or(self.network)
                .or(Some(Network::Bitcoin)),
            relay: self
                .relay
                .clone()
                .or(config_file.relay.clone())
                .or_else(|| {
                    Some(vec![
                        "wss://relay.damus.io".to_string(),
                        "wss://relay.snort.social".to_string(),
                    ])
                }),
            subcommand: self.subcommand.clone(),
        }
    }

    pub fn rpc_client(&self) -> anyhow::Result<bitcoincore_rpc::Client> {
        let host = self.rpchost.as_ref().ok_or(anyhow!("Missing RPC host"))?;
        let port = self.rpcport.ok_or(anyhow!("Missing RPC port"))?;
        let url = format!("{host}:{port}");
        let auth = if let Some(cookie) = &self.cookie {
            bitcoincore_rpc::Auth::CookieFile(cookie.clone())
        } else if self.rpcuser.is_some() && self.rpcpass.is_some() {
            bitcoincore_rpc::Auth::UserPass(
                self.rpcuser.clone().unwrap(),
                self.rpcpass.clone().unwrap(),
            )
        } else {
            bitcoincore_rpc::Auth::None
        };
        Ok(bitcoincore_rpc::Client::new(&url, auth)?)
    }

    pub async fn nostr_client(
        &self,
        sk: &str,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::from_sk_str(sk)?;
        let mut client =
            nostr_sdk::Client::new_with_opts(&keys, Options::new().wait_for_send(true));
        let relays = self.relay.clone().expect("No relays added");
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

#[derive(clap::Subcommand, Deserialize, Serialize, Debug, Clone)]
pub enum Subcommand {
    #[command(skip)]
    Noop,

    /// Generate a private/public keypair.
    GenerateKeypair,

    /// Create and broadcast new names.
    #[command(subcommand)]
    #[serde(skip)]
    New(NewSubcommand),

    /// Scan and index the blockchain.
    #[command(subcommand)]
    #[serde(skip)]
    Index(IndexSubcommand),

    /// Useful debugging commands
    #[command(subcommand)]
    #[serde(skip)]
    Debug(DebugSubcommand),
}

impl Default for Subcommand {
    fn default() -> Self {
        Subcommand::Noop
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum NewSubcommand {
    /// Create a new, unsigned transaction using a simple input document.
    /// Use `indigo new example` to create a sample document.
    Tx { document: PathBuf },

    /// Broadcast the new name transaction to Nostr relays.
    Broadcast {
        /// The same document used to create the name.
        document: PathBuf,

        /// Private key to sign the Nostr event
        privkey: String,
    },

    /// Print an example document for new names.
    Example,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum DebugSubcommand {
    ListNamespaces,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum IndexSubcommand {
    /// Index the blockchain and look for new namespaces.
    Blockchain {
        /// Minimum block confirmations for indexer
        #[arg(short, long, default_value = "3")]
        confirmations: usize,

        /// Starting block height to index. Do not specify to index from last indexed height.
        #[arg(long)]
        height: Option<usize>,
    },

    /// Query relays for missing namespace events.
    Relays,
}
