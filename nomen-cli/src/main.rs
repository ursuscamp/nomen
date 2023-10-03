#![allow(unused)]

mod nostr;

use clap::Parser;
use nomen_core::util::TransferBuilder;
use nostr::{Npub, Nsec};
use nostr_sdk::{Keys, ToBech32, UnsignedEvent};
use secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};

pub fn main() -> anyhow::Result<()> {
    let ops = Ops::parse();

    handle_ops(ops)?;

    Ok(())
}

fn handle_ops(ops: Ops) -> anyhow::Result<()> {
    match ops.command {
        Commands::Keys { pubkey, nostr } => cmd_keys(pubkey, nostr)?,
        Commands::Transfer { old, new, name } => cmd_transfer(old, new, name)?,
    }

    Ok(())
}

fn cmd_keys(pubkey: bool, nostr: bool) -> anyhow::Result<()> {
    let keys = nostr_sdk::Keys::generate();
    let (sk, pk) = if nostr {
        (
            keys.secret_key()?.to_bech32()?,
            keys.public_key().to_bech32()?,
        )
    } else {
        (
            keys.secret_key()?.display_secret().to_string(),
            keys.public_key().to_string(),
        )
    };
    println!("SK: {sk}");
    if pubkey {
        println!("PK: {pk}");
    }
    Ok(())
}

fn cmd_transfer(old: Nsec, new: Npub, name: String) -> anyhow::Result<()> {
    let tb = TransferBuilder {
        new_pubkey: new.as_ref(),
        name: &name,
    };
    let keys = nostr_sdk::Keys::new(*old.as_ref());
    let or1 = tb.transfer_op_return();
    let or2 = tb.signature_op_return(keys)?;
    println!("{}\n{}", hex::encode(or1), hex::encode(or2));
    Ok(())
}

#[derive(clap::Parser)]
struct Ops {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate Schnorr keypairs.
    Keys {
        #[arg(short, long)]
        pubkey: bool,

        #[arg(short, long)]
        nostr: bool,
    },

    /// Generate properly formatted OP_RETURNs for a name transfer.
    Transfer {
        /// Hex-encoded or bech32 (nsec) secret key for the current (previous) owner
        old: Nsec,

        /// Hex-encodced or bech32 (npub) public key for the new owner
        new: Npub,

        /// Name to transfer
        name: String,
    },
}
