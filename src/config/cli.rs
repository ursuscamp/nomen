use std::path::PathBuf;

use bitcoin::{
    address::{NetworkChecked, NetworkUnchecked},
    psbt::Psbt,
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

use crate::util::{KeyVal, Name, NomenKind, NostrSk};

use super::ConfigFile;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Location of config file: Default: nomen.toml
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

    /// Extra utilities
    #[command(subcommand)]
    Util(UtilSubcommand),

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
pub enum UtilSubcommand {
    /// Generate a private/public keypair.
    GenerateKeypair,

    /// Initialize a new config file.
    Init {
        /// Optional filename to write
        file: Option<PathBuf>,
    },

    /// Sign/broadcast a raw Nostr event
    SignEvent(SignEventCommand),

    /// Check if a name already exists
    Lookup {
        /// Name to look up
        name: String,
    },

    /// Generate the data to be used in an OP_RETURN.
    /// Useful when constructing transaction separately.
    OpReturn {
        /// The name to register
        name: String,

        /// The public key of the owner
        pubkey: XOnlyPublicKey,

        /// Transaction kind. Possible values: create, transfer
        kind: NomenKind,
    },
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
    // New(NameNewSubcommand),

    // TODO: pull this out to its own subcommand
    /// New from psbt
    New(NameNewSubcommand),

    /// Broadcast a new record for your name.
    Record(NameRecordSubcomand),

    /// Transfer a domain to a new keypair.
    Transfer(NameTransferSubcommand),
}

#[derive(clap::Args, Debug, Clone)]
pub struct NameNewSubcommand {
    /// New name to register.
    pub name: Name,

    /// The transaction to sign. May be a path to a PSBT file or a Base64 encoded PSBT string.
    pub psbt: String,

    /// The private key of the owner of the new name.
    #[arg(short, long)]
    pub privkey: Option<NostrSk>,

    /// Command output as JSON
    #[arg(short, long)]
    pub json: bool,

    /// Broadcast the associated Nostr event
    #[arg(short, long)]
    pub broadcast: bool,

    /// Verify against index that name is available.
    /// Be sure to run the indexer first, or this is not very useful.
    #[arg(short, long)]
    pub validate: bool,

    /// File path to write a serialized PSBT file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
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
    pub privkey: Option<NostrSk>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct NameTransferSubcommand {
    /// The name to broadcast records
    pub name: Name,

    /// Public key of the new owner
    pub pubkey: XOnlyPublicKey,

    /// The transaction to sign. May be a path to a PSBT file or a Base64 encoded PSBT string.
    pub psbt: String,

    /// Specify your private key on the command line. May be useful for scripts. Beware of shell history!
    /// Will prompt if not provided.
    /// This is the private key of the current owner of the name.
    #[arg(short, long)]
    pub privkey: Option<NostrSk>,

    /// JSON command output
    #[arg(short, long)]
    pub json: bool,

    /// Broadcast the associated Nostr event
    #[arg(short, long)]
    pub broadcast: bool,

    /// Verify against the index that the name is exists and is transferrable.
    /// Be sure to run the indexer first, or this is not very useful.
    #[arg(short, long)]
    pub validate: bool,

    /// File path to write a serialized PSBT file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
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
    pub privkey: Option<NostrSk>,

    /// Broadcast event to configured relays.
    #[arg(short, long)]
    pub broadcast: bool,

    pub event: String,
}
