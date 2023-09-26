use std::sync::atomic::{AtomicBool, Ordering};

use bitcoin::{BlockHash, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use futures::TryStreamExt;
use nomen_core::util::{NsidBuilder, SignatureV1, TransferBuilder};
use secp256k1::{schnorr::Signature, XOnlyPublicKey};
use sqlx::SqlitePool;

use crate::{
    config::{Cli, Config},
    db::{self, insert_index_height, BlockchainIndex},
    util::{CreateV0, CreateV1, NomenKind, Nsid, TransferV1},
};

enum QueueMessage {
    BlockchainIndex(BlockchainIndex),
    TransferCache(BlockchainIndex),
    TransferSignature(Signature),
}

pub async fn raw_index(
    config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), anyhow::Error> {
    // Check if the index is on a stale chain, and rewind the index if necessary
    rewind_invalid_chain(config.rpc_client()?, pool.clone()).await?;

    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool)
        .await?
        .max(config.starting_block_height());
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    log::info!("Scanning new blocks for indexable NOM outputs at height {index_height}");
    let min_confirmations = config.confirmations()?;

    let thread = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_header_info(&blockhash)?;

        loop {
            // If the channel is closed, let's stop
            if sender.is_closed() {
                log::info!("Stopping index operation.");
                break;
            }

            if (blockinfo.confirmations as usize) < min_confirmations {
                log::info!(
                    "Minimum confirmations not met at block height {}.",
                    blockinfo.height
                );
                break;
            }

            if blockinfo.height % 10 == 0 {
                log::info!("Index block height {}", blockinfo.height);
            }

            let block = client.get_block(&blockhash)?;

            for (txheight, tx) in block.txdata.iter().enumerate() {
                let mut transfer_cache: Option<BlockchainIndex> = None;
                for (vout, output) in tx.output.iter().enumerate() {
                    if output.script_pubkey.is_op_return() {
                        let b = &output.script_pubkey.as_bytes()[2..];

                        // Pre-check if it starts with NOM, so we can filter out some unnecessary errors from the logs
                        if b.starts_with(b"NOM") {
                            if let Ok(create) = CreateV0::try_from(b) {
                                let i = BlockchainIndex {
                                    protocol: 0,
                                    fingerprint: create.fingerprint,
                                    nsid: create.nsid,
                                    name: None,
                                    pubkey: None,
                                    blockhash,
                                    txid: tx.txid(),
                                    blocktime: blockinfo.time,
                                    blockheight: blockinfo.height,
                                    txheight,
                                    vout,
                                };
                                sender.blocking_send((
                                    (blockinfo.height, blockhash),
                                    Some(QueueMessage::BlockchainIndex(i)),
                                ));
                            } else if let Ok(create) = CreateV1::try_from(b) {
                                let i = BlockchainIndex {
                                    protocol: 1,
                                    fingerprint: create.fingerprint(),
                                    nsid: create.nsid(),
                                    name: Some(create.name),
                                    pubkey: Some(create.pubkey),
                                    blockhash,
                                    txid: tx.txid(),
                                    blocktime: blockinfo.time,
                                    blockheight: blockinfo.height,
                                    txheight,
                                    vout,
                                };
                                sender.blocking_send((
                                    (blockinfo.height, blockhash),
                                    Some(QueueMessage::BlockchainIndex(i)),
                                ));
                            } else if let Ok(transfer) = TransferV1::try_from(b) {
                                log::info!("Caching transfer for {}", transfer.name);
                                let i = BlockchainIndex {
                                    protocol: 1,
                                    fingerprint: transfer.fingerprint(),
                                    nsid: transfer.nsid(),
                                    name: Some(transfer.name),
                                    pubkey: Some(transfer.pubkey),
                                    blockhash,
                                    txid: tx.txid(),
                                    blocktime: blockinfo.time,
                                    blockheight: blockinfo.height,
                                    txheight,
                                    vout,
                                };
                                sender.blocking_send((
                                    (blockinfo.height, blockhash),
                                    Some(QueueMessage::TransferCache(i)),
                                ));
                            } else if let Ok(signature) = SignatureV1::try_from(b) {
                                log::info!("Signature found");
                                sender.blocking_send((
                                    (blockinfo.height, blockhash),
                                    Some(QueueMessage::TransferSignature(signature.signature)),
                                ));
                            } else {
                                log::error!("Index error");
                            }
                        } else {
                            sender.blocking_send(((blockinfo.height, blockhash), None));
                        }
                    } else {
                        sender.blocking_send(((blockinfo.height, blockhash), None));
                    }
                }
            }
            match blockinfo.next_block_hash {
                Some(next_hash) => {
                    blockhash = next_hash;
                    blockinfo = client.get_block_header_info(&blockhash)?;
                }
                None => break,
            }
        }

        Ok(())
    });

    let guard = elegant_departure::get_shutdown_guard();
    'select: loop {
        tokio::select! {
            msg = receiver.recv() => {
                match msg {
                    Some(((height, hash), Some(i))) => {
                        if let Err(e) = handle_message(
                            pool,
                            i
                        )
                        .await
                        {
                            log::error!("Index error: {e}");
                        }
                        insert_index_height(pool, height as i64, &hash).await?;
                    }
                    Some(((height, hash), None)) => {
                        insert_index_height(pool, height as i64, &hash).await?;
                    },
                    None => break 'select,
                }
            }
            _ = guard.wait() => {
                receiver.close();
                break 'select;
            }
        }
    }

    log::info!("Blockchain index complete.");
    Ok(())
}

async fn handle_message(conn: &SqlitePool, message: QueueMessage) -> anyhow::Result<()> {
    match message {
        QueueMessage::BlockchainIndex(index) => index_output(conn, index).await?,
        QueueMessage::TransferCache(index) => cache_transer(conn, index).await?,
        QueueMessage::TransferSignature(signature) => check_signature(conn, signature).await?,
    }

    Ok(())
}

async fn check_signature(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    signature: Signature,
) -> anyhow::Result<()> {
    let mut data = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT tc.name, tc.pubkey AS new_owner, n.pubkey, tc.id AS old_owner
        FROM transfer_cache tc
        JOIN valid_names_vw n ON tc.fingerprint = n.fingerprint AND tc.name = n.name",
    )
    .fetch(conn);

    while let Some(row) = data.try_next().await? {
        let name = row.0;
        let new_owner = {
            let h = hex::decode(row.1.as_bytes())?;
            XOnlyPublicKey::from_slice(&h)?
        };
        let old_owner = {
            let h = hex::decode(row.2.as_bytes())?;
            XOnlyPublicKey::from_slice(&h)?
        };
        let tb = TransferBuilder {
            new: &new_owner,
            name: name.as_str(),
        };
        let unsigned_event = tb.unsigned_event(&old_owner);
        if let Ok(event) = unsigned_event.add_signature(signature) {
            log::info!(
                "Valid signature found for {name}, updating owner to {}!",
                hex::encode(new_owner.serialize())
            );
            let nsid = NsidBuilder::new(name.as_str(), &new_owner).finalize();
            db::update_index_for_transfer(conn, nsid, new_owner, old_owner, name).await?;

            log::info!("Deleting record from transfer_cache");
            db::delete_from_transfer_cache(conn, row.3).await?;

            break;
        }
    }

    Ok(())
}

async fn index_output(conn: &SqlitePool, index: BlockchainIndex) -> anyhow::Result<()> {
    log::info!(
        "NOM output found: {}, name: {:?}, protocol: {}",
        index.nsid,
        index.name,
        index.protocol
    );

    // If we can verify that the v1 create is a valid v0 name that already exists, we can upgrade the v0 to the v1 automatically.
    if index.protocol == 1 {
        if let Some(name) = &index.name {
            if let Some(pubkey) = &index.pubkey {
                log::info!("Checking for upgrade");
                match db::upgrade_v0_to_v1(conn, name, *pubkey).await? {
                    db::UpgradeStatus::Upgraded => {
                        log::info!("Name '{name}' upgraded from v0 to v1.");
                    }
                    db::UpgradeStatus::NotUpgraded => {
                        log::info!("No upgrade found!");
                        db::insert_blockchain_index(conn, &index).await?;
                    }
                }
            }
        }
    } else {
        db::insert_blockchain_index(conn, &index).await?;
    }

    Ok(())
}

async fn cache_transer(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    index: BlockchainIndex,
) -> anyhow::Result<()> {
    db::insert_transfer_cache(conn, &index).await?;
    Ok(())
}

async fn rewind_invalid_chain(client: Client, pool: SqlitePool) -> anyhow::Result<()> {
    // Get the latest indexed blockhash and blockheight
    let result = sqlx::query_as::<_, (i32, String)>(
        "SELECT blockheight, blockhash FROM index_height ORDER BY blockheight DESC LIMIT 1;",
    )
    .fetch_optional(&pool)
    .await?;

    // No transactions indexed yet, skip the rest
    if result.is_none() {
        return Ok(());
    }

    let (blockheight, blockhash) = result.unwrap();

    // Loop backwards from recently indexed block, continuing to the previous block, until we find the most recent ancestor which is not stale
    let stale_block =
        tokio::task::spawn_blocking(move || -> Result<Option<usize>, anyhow::Error> {
            let mut next_block = Some(blockhash.parse()?);
            let mut stale_block = None;

            while let Some(next_blockhash) = next_block {
                let blockinfo = client.get_block_info(&next_blockhash)?;
                if blockinfo.confirmations >= 0 {
                    next_block = None;
                } else {
                    log::info!(
                        "Stale block {} detected at height {}",
                        blockinfo.hash,
                        blockinfo.height
                    );
                    stale_block = Some(blockinfo.height);
                    next_block = blockinfo.previousblockhash;
                }
            }

            Ok(stale_block)
        })
        .await??;

    // Delete entries from blockchain table
    if let Some(stale_block) = stale_block {
        log::info!("Reindexing beginning at height {stale_block}");
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM raw_blockchain WHERE blockheight >= ?;")
            .bind(stale_block as i32)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM blockchain WHERE blockheight >= ?;")
            .bind(stale_block as i32)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM index_height WHERE blockheight >= ?;")
            .bind(stale_block as i32)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}
