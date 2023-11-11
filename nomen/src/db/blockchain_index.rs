use bitcoin::{BlockHash, Txid};
use nomen_core::Nsid;
use secp256k1::XOnlyPublicKey;
use sqlx::{Executor, Sqlite};

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
