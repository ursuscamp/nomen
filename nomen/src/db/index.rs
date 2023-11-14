#![allow(clippy::module_name_repetitions)]

use bitcoin::{BlockHash, Txid};
use nomen_core::{Hash160, Nsid, NsidBuilder};
use secp256k1::XOnlyPublicKey;
use sqlx::{Executor, Sqlite, SqlitePool};

pub struct BlockchainIndex {
    pub protocol: i64,
    pub fingerprint: [u8; 5],
    pub nsid: Nsid,
    pub name: Option<String>,
    pub pubkey: Option<XOnlyPublicKey>,
    pub blockhash: BlockHash,
    pub txid: Txid,
    pub blocktime: usize,
    pub blockheight: usize,
    pub txheight: usize,
    pub vout: usize,
}

pub async fn insert_blockchain_index(
    conn: impl Executor<'_, Database = Sqlite>,
    index: &BlockchainIndex,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_blockchain_index.sql"))
        .bind(index.protocol)
        .bind(hex::encode(index.fingerprint))
        .bind(index.nsid.to_string())
        .bind(&index.name)
        .bind(index.pubkey.map(|k| k.to_string()))
        .bind(&index.blockhash.to_string())
        .bind(index.txid.to_string())
        .bind(index.blocktime as i64)
        .bind(index.blockheight as i64)
        .bind(index.txheight as i64)
        .bind(index.vout as i64)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn insert_transfer_cache(
    conn: impl Executor<'_, Database = Sqlite>,
    index: &BlockchainIndex,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_transfer_cache.sql"))
        .bind(index.protocol)
        .bind(hex::encode(index.fingerprint))
        .bind(index.nsid.to_string())
        .bind(&index.name)
        .bind(index.pubkey.map(|k| k.to_string()))
        .bind(&index.blockhash.to_string())
        .bind(index.txid.to_string())
        .bind(index.blocktime as i64)
        .bind(index.blockheight as i64)
        .bind(index.txheight as i64)
        .bind(index.vout as i64)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn next_index_height(conn: &SqlitePool) -> anyhow::Result<usize> {
    let (h,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(blockheight), 0) + 1 FROM index_height;")
            .fetch_one(conn)
            .await?;

    Ok(h as usize)
}

pub async fn insert_height(
    conn: &SqlitePool,
    height: i64,
    blockhash: &BlockHash,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO index_height (blockheight, blockhash) VALUES (?, ?) ON CONFLICT DO NOTHING;",
    )
    .bind(height)
    .bind(blockhash.to_string())
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn update_for_transfer(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    nsid: Nsid,
    new_owner: XOnlyPublicKey,
    old_owner: XOnlyPublicKey,
    name: String,
) -> Result<(), anyhow::Error> {
    sqlx::query("UPDATE blockchain_index SET nsid = ?, pubkey = ? WHERE name = ? AND pubkey = ?;")
        .bind(hex::encode(nsid.as_ref()))
        .bind(hex::encode(new_owner.serialize()))
        .bind(&name)
        .bind(hex::encode(old_owner.serialize()))
        .execute(conn)
        .await?;
    Ok(())
}

pub enum UpgradeStatus {
    Upgraded,
    NotUpgraded,
}

pub async fn upgrade_v0_to_v1(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
    pubkey: XOnlyPublicKey,
    blockheight: usize,
    txid: Txid,
) -> anyhow::Result<UpgradeStatus> {
    let fingerprint = hex::encode(
        Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint(),
    );
    let nsid = hex::encode(NsidBuilder::new(name, &pubkey).finalize().as_ref());

    let updated = sqlx::query(
        "UPDATE blockchain_index
        SET name = ?, pubkey = ?, protocol = 1, v1_upgrade_blockheight = ?, v1_upgrade_txid = ?
        WHERE fingerprint = ? AND nsid = ? AND protocol = 0;",
    )
    .bind(name)
    .bind(hex::encode(pubkey.serialize()))
    .bind(blockheight as i64)
    .bind(hex::encode(txid))
    .bind(&fingerprint)
    .bind(&nsid)
    .execute(conn)
    .await?;

    if updated.rows_affected() > 0 {
        return Ok(UpgradeStatus::Upgraded);
    }

    Ok(UpgradeStatus::NotUpgraded)
}

pub async fn update_v0_index(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
    pubkey: &XOnlyPublicKey,
    nsid: Nsid,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE blockchain_index SET name = ?, pubkey = ? WHERE protocol = 0 AND nsid = ?;",
    )
    .bind(name)
    .bind(pubkey.to_string())
    .bind(hex::encode(nsid.as_slice()))
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn delete_from_transfer_cache(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    id: i64,
) -> Result<(), anyhow::Error> {
    tracing::debug!("DELETING transfer_cache with id {id}");
    sqlx::query("DELETE FROM transfer_cache WHERE id = ?;")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn reindex(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    blockheight: i64,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM blockchain_index WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(conn)
        .await?;
    sqlx::query("DELETE FROM transfer_cache WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(conn)
        .await?;
    sqlx::query("DELETE FROM old_transfer_cache WHERE blockheight >= ?;")
        .bind(blockheight)
        .execute(conn)
        .await?;
    sqlx::query("DELETE FROM name_events;")
        .execute(conn)
        .await?;
    Ok(())
}
