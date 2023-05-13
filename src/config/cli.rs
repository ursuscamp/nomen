use std::path::PathBuf;

use bitcoin::{
    address::{NetworkChecked, NetworkUnchecked},
    secp256k1::SecretKey,
    Network,
};
use clap::Parser;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Options,
};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite, SqlitePool};

use crate::util::{KeyVal, Name};

use super::ConfigFile;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Location of config file: Default: .nomen.toml
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path for index data. Default: nomen.db
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

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
    #[command(skip)]
    Noop,

    /// Generate a private/public keypair.
    GenerateKeypair,

    /// Initialize a new config file.
    Init {
        /// Optional filename to write
        file: Option<PathBuf>,
    },

    /// Sign/broadcast a raw Nostr event
    SignEvent(SignEventCommand),

    /// Create and broadcast new names.
    #[command(subcommand)]
    Name(Box<NameSubcommand>),

    /// Scan and index the blockchain.
    Index,

    /// Start the HTTP server
    Server(ServerSubcommand),
}

impl Default for Subcommand {
    fn default() -> Self {
        Subcommand::Noop
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum IndexSubcommand {
    /// Index the blockchain and look for new namespaces.
    Blockchain {
        /// Minimum block confirmations for indexer. Default: 3
        #[arg(short, long)]
        confirmations: Option<usize>,

        /// Starting block height to index. Default: most recently scanned block
        #[arg(long)]
        height: Option<usize>,
    },

    /// Query relays for missing create events.
    CreateEvents,

    /// Query relays for records
    RecordsEvents,
}

#[derive(clap::Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerSubcommand {
    /// Address and port to bind.
    #[arg(short, long)]
    pub bind: Option<String>,

    /// Start server without explorer.
    #[arg(long)]
    pub without_explorer: bool,

    /// Start server without API.
    #[arg(long)]
    pub without_api: bool,

    /// Start server without indexer.
    #[arg(long)]
    pub without_indexer: bool,

    /// Delay (in seconds) between indexing operations.
    #[arg(long)]
    pub indexer_delay: Option<u64>,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum NameSubcommand {
    /// Create a new name.
    New(NameNewSubcommand),

    /// Broadcast a new record for your name.
    Record(NameRecordSubcomand),

    /// Transfer a domain to a new keypair.
    Transfer(NameTransferSubcommand),
}

#[derive(clap::Args, Debug, Clone)]
pub struct NameNewSubcommand {
    /// The root name of the new namespace.
    pub name: Name,

    #[command(flatten)]
    pub txinfo: TxInfo,

    /// Specify your private key on the command line. May be useful for scripts. Beware of shell history!
    /// Will prompt if not provided.
    #[arg(short, long)]
    pub privkey: Option<SecretKey>,

    /// JSON command output
    #[arg(short, long)]
    pub json: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct NameRecordSubcomand {
    /// The name you are broadcasting records for
    pub name: Name,

    /// Records to broadcast (format "key=value")
    pub records: Vec<KeyVal>,

    /// Specify your private key on the command line. May be useful for scripts. Beware of shell history!
    /// Will prompt if not provided.
    #[arg(short, long)]
    pub privkey: Option<SecretKey>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct NameTransferSubcommand {
    /// The name to be transferred.
    pub name: Name,

    /// The public key of the previous owner.
    pub previous: XOnlyPublicKey,

    /// The public key of the new owner.
    pub new: XOnlyPublicKey,

    #[command(flatten)]
    pub txinfo: TxInfo,

    /// JSON command output
    #[arg(short, long)]
    pub json: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct TxInfo {
    /// The txid to use as input.
    pub txid: bitcoin::Txid,

    /// Tx output number to use as input.
    pub vout: u32,

    /// New address to send outputs
    pub address: bitcoin::Address<NetworkUnchecked>,

    /// Fee to use for the transaction (sats/vb)
    #[arg(short, long, default_value = "1")]
    pub fee: u64,
}

#[derive(clap::Args, Debug, Clone)]
pub struct SignEventCommand {
    /// Specify your private key on the command line. May be useful for scripts. Beware of shell history!
    /// Will prompt if not provided.
    #[arg(short, long)]
    pub privkey: Option<SecretKey>,

    /// Broadcast event to configured relays.
    #[arg(short, long)]
    pub broadcast: bool,

    pub event: String,
}
