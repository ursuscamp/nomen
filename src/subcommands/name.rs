use std::io::Write;

use bitcoin::secp256k1::SecretKey;
use nostr_sdk::Keys;

use crate::{
    config::{Config, NameSubcommand},
    hash160::Hash160,
};

pub async fn name(config: &Config, cmd: &NameSubcommand) -> anyhow::Result<()> {
    match cmd {
        NameSubcommand::New(new_data) => new::new(config, new_data).await?,
        NameSubcommand::Record(record_data) => record::record(config, record_data).await?,
    }

    Ok(())
}

mod new {
    use anyhow::anyhow;
    use bitcoin::hashes::hex::ToHex;
    use bitcoincore_rpc::{RawTx, RpcApi};
    use nostr_sdk::{prelude::TagKind, EventBuilder, Keys, Tag};

    use crate::{
        config::{Config, NameNewSubcommand},
        hash160::Hash160,
        subcommands::name::{nsid, parse_keys},
        util::NamespaceNostrKind,
    };

    pub async fn new(config: &Config, args: &NameNewSubcommand) -> anyhow::Result<()> {
        let keys = parse_keys(&args.privkey)?;
        let children = parse_children(&args.children)?;
        let mr = children_merkle_root(&children, &args.name)?;
        let nsid = nsid(&args.name, mr.as_ref(), &keys);
        let tx = create_unsigned_tx(config, args, &nsid).await?;

        println!("Nsid: {}", nsid.to_hex());
        println!("Unsigned Tx: {}", tx.raw_hex());

        let event = create_event(children, nsid, args, keys)?;
        let (_k, nostr) = config.nostr_random_client().await?;
        let event_id = nostr.send_event(event).await?;

        println!("Sent event {event_id}");

        Ok(())
    }

    fn create_event(
        children: Vec<(String, Vec<u8>)>,
        nsid: Vec<u8>,
        args: &NameNewSubcommand,
        keys: Keys,
    ) -> Result<nostr_sdk::Event, anyhow::Error> {
        let children_json = {
            let s = children
                .into_iter()
                .map(|(name, pubkey)| (name, pubkey.to_hex()))
                .collect::<Vec<_>>();
            serde_json::to_string(&s)
        }?;
        let event = EventBuilder::new(
            NamespaceNostrKind::Name.into(),
            children_json,
            &[
                Tag::Identifier(nsid.to_hex()),
                Tag::Generic(TagKind::Custom("ind".to_owned()), vec![args.name.clone()]),
            ],
        )
        .to_event(&keys)?;
        Ok(event)
    }

    async fn create_unsigned_tx(
        config: &Config,
        args: &NameNewSubcommand,
        nsid: &[u8],
    ) -> Result<bitcoin::Transaction, anyhow::Error> {
        let tx = get_transaction(config, &args.txid).await?;
        let txout = &tx.output[args.vout as usize];
        let new_amount = txout
            .value
            .checked_sub(args.fee as u64)
            .ok_or_else(|| anyhow!("Fee is over available amount in tx"))?;
        let txin = bitcoin::TxIn {
            previous_output: bitcoin::OutPoint {
                txid: args.txid,
                vout: args.vout,
            },
            script_sig: bitcoin::Script::new(), // Unsigned tx with empty script
            sequence: bitcoin::Sequence::ZERO,
            witness: bitcoin::Witness::new(),
        };
        let txout = bitcoin::TxOut {
            value: new_amount,
            script_pubkey: args.address.script_pubkey(),
        };
        let op_return = bitcoin::TxOut {
            value: 0,
            script_pubkey: bitcoin::Script::new_op_return(&op_return(nsid)),
        };
        let tx = bitcoin::Transaction {
            version: 1,
            lock_time: bitcoin::PackedLockTime::ZERO,
            input: vec![txin],
            output: vec![txout, op_return],
        };
        Ok(tx)
    }

    fn op_return(nsid: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(30);
        v.extend(b"IND\x00\x00");
        v.extend(nsid);
        v
    }

    async fn get_transaction(
        config: &Config,
        txid: &bitcoin::Txid,
    ) -> Result<bitcoin::Transaction, anyhow::Error> {
        let client = config.rpc_client()?;
        let txid = *txid;
        Ok(tokio::task::spawn_blocking(move || client.get_raw_transaction(&txid, None)).await??)
    }

    fn children_merkle_root(
        children: &[(String, Vec<u8>)],
        root: &str,
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        let child_hashes = child_hashes(children, root);
        let mr = if children.is_empty() {
            None
        } else {
            Some(merkle_root(&child_hashes))
        };
        Ok(mr)
    }

    fn merkle_root(child_hashes: &[Vec<u8>]) -> Vec<u8> {
        let mut queue = child_hashes.to_vec();
        if queue.len() % 2 != 0 {
            queue.push(
                queue
                    .last()
                    .cloned()
                    .expect("merkle_root expects at least one item"),
            );
        }

        while queue.len() > 1 {
            queue = queue
                .chunks(2)
                .map(|chunk| Hash160::digest_slices(&[chunk[0].as_ref(), chunk[1].as_ref()]).into())
                .collect();
        }

        queue.first().cloned().unwrap()
    }

    // TODO: change to use nsid builder
    fn child_hashes(children: &[(String, Vec<u8>)], root: &str) -> Vec<Vec<u8>> {
        children
            .iter()
            .map(|(n, pk)| {
                let root_end = format!(".{root}");
                if n.ends_with(&root_end) {
                    (n.clone(), pk)
                } else {
                    (format!("{n}{root_end}"), pk)
                }
            })
            .map(|(n, pk)| {
                let mut hash160 = Hash160::default();
                hash160.update(n.as_bytes());
                hash160.update(pk);
                hash160.finalize().to_vec()
            })
            .collect()
    }

    fn parse_children(children: &[String]) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        children
            .iter()
            .map(|child| -> anyhow::Result<(String, Vec<u8>)> {
                let mut splitter = child.split(':');
                let name = splitter
                    .next()
                    .ok_or_else(|| anyhow!("Unparseable child name"))?;
                let pk = splitter
                    .next()
                    .ok_or_else(|| anyhow!("Unparseable child pubkey"))?;
                let pk = hex::decode(pk)?;
                Ok((name.to_lowercase(), pk))
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use std::str::FromStr;

        use bitcoin::secp256k1::SecretKey;

        use super::*;

        #[test]
        fn test_merkle_root() {
            let sk = "f5daf17ccf02488bc0ab506fc550016963af3030d4c5d2b7b3e3c232f3c0d7ca";
            let keys = Keys::new(SecretKey::from_str(sk).unwrap());
            let ch = child_hashes(
                &[
                    ("bob".to_string(), keys.public_key().serialize().to_vec()),
                    ("alice".to_string(), keys.public_key().serialize().to_vec()),
                ],
                "smith",
            );
            let mr = merkle_root(&ch);

            assert_eq!(mr.to_hex(), "08c6edfa1e25e57c67bf19294071df22e29e826a")
        }
    }
}

mod record {
    use std::collections::HashMap;

    use nostr_sdk::{prelude::TagKind, EventBuilder, Tag};

    use crate::{
        config::{Config, NameRecordSubcomand},
        util::NamespaceNostrKind,
    };

    use super::parse_keys;

    pub async fn record(config: &Config, record_data: &NameRecordSubcomand) -> anyhow::Result<()> {
        let keys = parse_keys(&record_data.privkey)?;
        let map = parse_records(&record_data.records);
        let records = serde_json::to_string(&map)?;

        let event = EventBuilder::new(
            NamespaceNostrKind::Record.into(),
            records,
            &[
                Tag::Identifier(record_data.nsid.clone()),
                Tag::Generic(
                    TagKind::Custom("ind".to_owned()),
                    vec![record_data.name.clone()],
                ),
            ],
        )
        .to_event(&keys)?;

        let (_keys, client) = config.nostr_random_client().await?;
        let event_id = client.send_event(event).await?;
        println!("Sent event {event_id}");

        Ok(())
    }

    fn parse_records(records: &[String]) -> HashMap<String, String> {
        records
            .iter()
            .filter_map(|rec| rec.split_once('='))
            .map(|(k, v)| (k.to_uppercase(), v.to_owned()))
            .collect()
    }
}

fn parse_keys(privkey: &Option<SecretKey>) -> Result<Keys, anyhow::Error> {
    let privkey = if let Some(s) = privkey {
        *s
    } else {
        // TODO: use a better system for getting secure info than this, like a secure prompt
        print!("Private key: ");
        std::io::stdout().flush()?;
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        s.trim().to_string().parse()?
    };
    let keys = Keys::new(privkey);
    Ok(keys)
}

fn nsid(name: &str, mr: Option<&Vec<u8>>, keys: &Keys) -> Vec<u8> {
    let mut hasher = Hash160::default();
    hasher.update(name.as_bytes());
    if let Some(mr) = mr {
        hasher.update(mr);
    }
    hasher.update(&keys.public_key().serialize());
    hasher.finalize().to_vec()
}
