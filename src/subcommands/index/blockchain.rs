use std::sync::atomic::{AtomicBool, Ordering};

use bitcoin::{BlockHash, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use sqlx::SqlitePool;

use crate::{
    config::{Cli, Config},
    db::{self, insert_index_height},
    util::{NomenKind, NomenTx, Nsid},
};

pub async fn index(config: &Config, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), anyhow::Error> {
    // Check if the index is on a stale chain, and rewind the index if necessary
    rewind_invalid_chain(config.rpc_client()?, pool.clone()).await?;

    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool)
        .await?
        .max(config.starting_block_height());
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    log::info!("Starting blockchain index at height {index_height}");
    let min_confirmations = config.confirmations()?;

    let thread = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_header_info(&blockhash)?;

        while let Some(next_hash) = blockinfo.next_block_hash {
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
                for (vout, output) in tx.output.iter().enumerate() {
                    if output.script_pubkey.is_op_return() {
                        let b = &output.script_pubkey.as_bytes()[2..];

                        // Pre-check if it starts with NOM, so we can filter out some unnecessary errors from the logs
                        if b.starts_with(b"NOM") {
                            match NomenTx::try_from(b) {
                                Ok(NomenTx {
                                    fingerprint,
                                    nsid,
                                    kind,
                                }) => {
                                    sender.blocking_send((
                                        (blockinfo.height, blockhash),
                                        Some((
                                            fingerprint,
                                            nsid,
                                            blockhash,
                                            tx.txid(),
                                            blockinfo.time,
                                            blockinfo.height,
                                            txheight,
                                            vout,
                                            kind,
                                        )),
                                    ));
                                }

                                Err(e) => log::error!("Index error: {e}"),
                            }
                        } else {
                            sender.blocking_send(((blockinfo.height, blockhash), None));
                        }
                    }
                }
            }
            blockhash = next_hash;
            blockinfo = client.get_block_header_info(&blockhash)?;
        }

        Ok(())
    });

    let guard = elegant_departure::get_shutdown_guard();
    'select: loop {
        tokio::select! {
            msg = receiver.recv() => {
                match msg {
                    Some(((height, hash), Some((fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, kind)))) => {
                        if let Err(e) = index_output(
                            pool,
                            fingerprint,
                            nsid,
                            &blockhash,
                            &txid,
                            blocktime,
                            blockheight,
                            txheight,
                            vout,
                            kind,
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

#[allow(clippy::too_many_arguments)]
async fn index_output(
    conn: &SqlitePool,
    fingerprint: [u8; 5],
    nsid: Nsid,
    blockhash: &BlockHash,
    txid: &Txid,
    blocktime: usize,
    blockheight: usize,
    txheight: usize,
    vout: usize,
    kind: NomenKind,
) -> anyhow::Result<()> {
    log::info!("NOM output found: {}", nsid);
    if nsid.len() != 20 {
        return Err(anyhow::anyhow!("Unexpected NOM length"));
    }

    db::insert_blockchain(
        conn,
        fingerprint,
        nsid,
        blockhash.to_string(),
        txid.to_string(),
        blocktime,
        blockheight,
        txheight,
        vout,
        kind,
    )
    .await?;
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
