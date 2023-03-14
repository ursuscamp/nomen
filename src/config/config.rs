use std::path::PathBuf;

use bitcoin::Network;
use clap::Parser;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Options,
};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;

use super::ConfigFile;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    /// Location of config file: Default: .indigo.toml
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path for index data. Default: indigo.db
    #[arg(short, long)]
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
    pub relays: Option<Vec<String>>,

    #[command(subcommand)]
    pub subcommand: Subcommand,
}

impl Config {
    pub fn merge_config_file(&mut self, cf: ConfigFile) {
        self.data = self
            .data
            .take()
            .or(cf.data)
            .or_else(|| Some("indigo.db".into()));
        self.cookie = self.cookie.take().or(cf.cookie);
        self.rpcuser = self.rpcuser.take().or(cf.rpcuser);
        self.rpcpass = self.rpcpass.take().or(cf.rpcpass);
        self.rpchost = self.rpchost.take().or(cf.rpchost);
        self.rpcport = self.rpcport.or(cf.rpcport);
        self.network = self.network.or(cf.network);
        self.relays = self.relays.take().or(cf.relays);
        if let Subcommand::Server { bind } = &mut self.subcommand {
            *bind = bind
                .take()
                .or(cf.server.map(|mut s| s.bind.take()).flatten())
                .or_else(|| Some("0.0.0.0:8080".into()));
        }
    }

    pub fn rpc_auth(&self) -> bitcoincore_rpc::Auth {
        if let Some(cookie) = &self.cookie {
            bitcoincore_rpc::Auth::CookieFile(cookie.clone())
        } else if self.rpcuser.is_some() || self.rpcpass.is_some() {
            bitcoincore_rpc::Auth::UserPass(
                self.rpcuser.clone().expect("RPC user not configured"),
                self.rpcpass.clone().expect("RPC password not configured"),
            )
        } else {
            bitcoincore_rpc::Auth::None
        }
    }

    pub fn rpc_client(&self) -> anyhow::Result<bitcoincore_rpc::Client> {
        let host = self.rpchost.clone().unwrap_or_else(|| "127.0.0.1".into());
        let port = self.rpcport.unwrap_or(8332);
        let url = format!("{host}:{port}");
        let auth = self.rpc_auth();
        Ok(bitcoincore_rpc::Client::new(&url, auth)?)
    }

    pub async fn sqlite(&self) -> anyhow::Result<tokio_rusqlite::Connection> {
        Ok(Connection::open(&self.data.as_ref().expect("No database configured")).await?)
    }

    pub async fn nostr_client(
        &self,
        sk: &str,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::from_sk_str(sk)?;
        let mut client =
            nostr_sdk::Client::new_with_opts(&keys, Options::new().wait_for_send(true));
        let relays = self.relays.as_ref().expect("No relays configured");
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

    /// Broadcast records updates
    #[command(subcommand)]
    #[serde(skip)]
    Records(RecordsSubcommand),

    /// Scan and index the blockchain.
    #[command(subcommand)]
    #[serde(skip)]
    Index(IndexSubcommand),

    /// Useful debugging commands
    #[command(subcommand)]
    #[serde(skip)]
    Debug(DebugSubcommand),

    /// Start the HTTP server
    Server {
        #[arg(short, long)]
        bind: Option<String>,
    },
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
pub enum RecordsSubcommand {
    /// Broadcast a records event to the relays.
    Broadcast {
        /// A document representing the records to be relayed.
        document: PathBuf,

        /// Private key to sign the Nostr event.
        privkey: String,
    },

    /// Print an example document to update the name.
    Example,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum DebugSubcommand {
    ListNamespaces,
    NamesIndex,
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

    /// Query relays for missing create events.
    CreateEvents,

    /// Query relays for records
    RecordsEvents,
}
