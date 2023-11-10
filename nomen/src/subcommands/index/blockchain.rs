use bitcoin::BlockHash;
use bitcoincore_rpc::{Client, RpcApi};
use futures::TryStreamExt;
use nomen_core::{CreateV0, CreateV1, NsidBuilder, SignatureV1, TransferBuilder, TransferV1};
use secp256k1::{schnorr::Signature, XOnlyPublicKey};
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db::{self, insert_index_height, BlockchainIndex, RawBlockchain},
};

enum QueueMessage {
    RawBlockchain(RawBlockchain),
    Index {
        blockheight: i64,
        blockhash: BlockHash,
    },
}

pub async fn index(config: &Config, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), anyhow::Error> {
    // Check if the index is on a stale chain, and rewind the index if necessary
    rewind_invalid_chain(config.rpc_client()?, pool.clone()).await?;

    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool)
        .await?
        .max(config.starting_block_height());
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    tracing::info!("Scanning new blocks for indexable NOM outputs at height {index_height}");
    let min_confirmations = config.confirmations();

    // Spawn a thread to query the Bitcoin node for new block data. Messages are sent to the queue.
    let _thread = spawn_index_thread(client, index_height, sender, min_confirmations);

    // Process the messages from the queue. This will push new NOM OP_RETURNs into the raw_blockchain table.
    process_messages(receiver, pool).await?;

    // Update the blockchain index by looping through raw_blockchain table and pocessing the saved outputs.
    update_blockchain_index(config, pool).await?;

    // Expire unused transfer cache
    expire_transfer_cache(pool).await?;

    tracing::info!("Blockchain index complete.");
    Ok(())
}

async fn process_messages(
    mut receiver: tokio::sync::mpsc::Receiver<QueueMessage>,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> anyhow::Result<()> {
    let guard = elegant_departure::get_shutdown_guard();
    'select: loop {
        tokio::select! {
            msg = receiver.recv() => {
                match msg {
                    Some(QueueMessage::RawBlockchain(raw_blockchain)) => {
                        if let Err(e) = db::insert_raw_blockchain(pool, &raw_blockchain)
                        .await
                        {
                            tracing::error!("Index error: {e}");
                        }
                        insert_index_height(pool, raw_blockchain.blockheight as i64, &raw_blockchain.blockhash).await?;
                    }
                    Some(QueueMessage::Index {blockheight, blockhash}) => {
                        insert_index_height(pool, blockheight, &blockhash).await?;
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
    Ok(())
}

fn spawn_index_thread(
    client: Client,
    index_height: usize,
    sender: tokio::sync::mpsc::Sender<QueueMessage>,
    min_confirmations: usize,
) -> tokio::task::JoinHandle<Result<(), anyhow::Error>> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_header_info(&blockhash)?;

        loop {
            // If the channel is closed, let's stop
            if sender.is_closed() {
                tracing::info!("Stopping index operation.");
                break;
            }

            if (blockinfo.confirmations as usize) < min_confirmations {
                tracing::info!(
                    "Minimum confirmations not met at block height {}.",
                    blockinfo.height
                );
                break;
            }

            if blockinfo.height % 10 == 0 {
                tracing::info!("Index block height {}", blockinfo.height);
            }

            let block = client.get_block(&blockhash)?;

            for (txheight, tx) in block.txdata.iter().enumerate() {
                for (vout, output) in tx.output.iter().enumerate() {
                    if output.script_pubkey.is_op_return() && output.script_pubkey.len() >= 3 {
                        let b = &output.script_pubkey.as_bytes()[2..];

                        // Pre-check if it starts with NOM, so we can filter out some unnecessary errors from the logs
                        if b.starts_with(b"NOM") {
                            let raw_blockchain = RawBlockchain {
                                blockhash,
                                txid: tx.txid(),
                                blocktime: blockinfo.time,
                                blockheight: blockinfo.height,
                                txheight,
                                vout,
                                data: b.to_vec(),
                            };
                            sender
                                .blocking_send(QueueMessage::RawBlockchain(raw_blockchain))
                                .ok();
                        } else {
                            sender
                                .blocking_send(QueueMessage::Index {
                                    blockheight: blockinfo.height as i64,
                                    blockhash,
                                })
                                .ok();
                        }
                    } else {
                        sender
                            .blocking_send(QueueMessage::Index {
                                blockheight: blockinfo.height as i64,
                                blockhash,
                            })
                            .ok();
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
    })
}

pub async fn update_blockchain_index(
    _config: &Config,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), anyhow::Error> {
    let rows = sqlx::query_as::<_, RawBlockchain>("SELECT * FROM raw_blockchain rb WHERE rb.blockheight > (SELECT coalesce(max(blockheight), 0) FROM index_blockheights_vw);").fetch_all(pool).await?;
    for row in rows {
        if let Ok(create) = CreateV0::try_from(row.data.as_ref()) {
            let i = BlockchainIndex {
                protocol: 0,
                fingerprint: create.fingerprint,
                nsid: create.nsid,
                name: None,
                pubkey: None,
                blockhash: row.blockhash,
                txid: row.txid,
                blocktime: row.blocktime,
                blockheight: row.blockheight,
                txheight: row.txheight,
                vout: row.vout,
            };
            index_output(pool, i).await?;
        } else if let Ok(create) = CreateV1::try_from(row.data.as_ref()) {
            let i = BlockchainIndex {
                protocol: 1,
                fingerprint: create.fingerprint(),
                nsid: create.nsid(),
                name: Some(create.name),
                pubkey: Some(create.pubkey),
                blockhash: row.blockhash,
                txid: row.txid,
                blocktime: row.blocktime,
                blockheight: row.blockheight,
                txheight: row.txheight,
                vout: row.vout,
            };
            index_output(pool, i).await?;
        } else if let Ok(transfer) = TransferV1::try_from(row.data.as_ref()) {
            tracing::info!("Caching transfer for {}", transfer.name);
            let i = BlockchainIndex {
                protocol: 1,
                fingerprint: transfer.fingerprint(),
                nsid: transfer.nsid(),
                name: Some(transfer.name),
                pubkey: Some(transfer.pubkey),
                blockhash: row.blockhash,
                txid: row.txid,
                blocktime: row.blocktime,
                blockheight: row.blockheight,
                txheight: row.txheight,
                vout: row.vout,
            };
            cache_transfer(pool, i).await?;
        } else if let Ok(signature) = SignatureV1::try_from(row.data.as_ref()) {
            tracing::info!("Signature found");
            check_signature(pool, signature.signature).await?;
        } else {
            tracing::error!("Index error");
        }
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
            new_pubkey: &new_owner,
            name: name.as_str(),
        };
        let unsigned_event = tb.unsigned_event(&old_owner);
        if unsigned_event.add_signature(signature).is_ok() {
            tracing::info!(
                "Valid signature found for {name}, updating owner to {}!",
                hex::encode(new_owner.serialize())
            );
            let nsid = NsidBuilder::new(name.as_str(), &new_owner).finalize();
            db::update_index_for_transfer(conn, nsid, new_owner, old_owner, name).await?;

            tracing::info!("Deleting record from transfer_cache");
            db::delete_from_transfer_cache(conn, row.3).await?;

            break;
        }
    }

    Ok(())
}

async fn index_output(conn: &SqlitePool, index: BlockchainIndex) -> anyhow::Result<()> {
    tracing::info!(
        "NOM output found: {}, name: {:?}, protocol: {}",
        index.nsid,
        index.name,
        index.protocol
    );

    // If we can verify that the v1 create is a valid v0 name that already exists, we can upgrade the v0 to the v1 automatically.
    if index.protocol == 1 {
        if let Some(name) = &index.name {
            if let Some(pubkey) = &index.pubkey {
                tracing::info!("Checking for upgrade");
                match db::upgrade_v0_to_v1(conn, name, *pubkey).await? {
                    db::UpgradeStatus::Upgraded => {
                        tracing::info!("Name '{name}' upgraded from v0 to v1.");
                    }
                    db::UpgradeStatus::NotUpgraded => {
                        tracing::info!("No upgrade found!");
                        db::insert_blockchain_index(conn, &index).await?;
                    }
                }
            }
            db::relay_index::queue(conn, name).await?;
        }
    } else {
        db::insert_blockchain_index(conn, &index).await?;
    }

    Ok(())
}

async fn cache_transfer(
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

    let (_, blockhash) = result.unwrap();

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
                    tracing::info!(
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
        tracing::info!("Reindexing beginning at height {stale_block}");
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM raw_blockchain WHERE blockheight >= ?;")
            .bind(stale_block as i64)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM blockchain_index WHERE blockheight >= ?;")
            .bind(stale_block as i64)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM transfer_cache WHERE blockheight >= ?;")
            .bind(stale_block as i64)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM old_transfer_cache WHERE blockheight >= ?;")
            .bind(stale_block as i64)
            .execute(&mut tx)
            .await?;
        sqlx::query("DELETE FROM index_height WHERE blockheight >= ?;")
            .bind(stale_block as i64)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}

async fn expire_transfer_cache(pool: &sqlx::Pool<sqlx::Sqlite>) -> anyhow::Result<()> {
    tracing::info!("Starting transfer cache expiration.");
    let (index_height,) = sqlx::query_as::<_, (i64,)>("SELECT max(blockheight) FROM index_height;")
        .fetch_one(pool)
        .await?;
    sqlx::query("INSERT INTO old_transfer_cache SELECT * FROM transfer_cache WHERE blockheight < (? - 100);").bind(index_height).execute(pool).await?;
    sqlx::query("DELETE FROM transfer_cache WHERE blockheight < (? - 100);")
        .bind(index_height)
        .execute(pool)
        .await?;
    tracing::info!("Finished transfer cache expiration.");
    Ok(())
}
