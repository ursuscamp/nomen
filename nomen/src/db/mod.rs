use std::{collections::HashMap, str::FromStr};

use bitcoin::{BlockHash, Txid};
use nomen_core::{Hash160, Name, Nsid, NsidBuilder};
use nostr_sdk::EventId;
use secp256k1::XOnlyPublicKey;
use sqlx::{sqlite::SqliteRow, Executor, FromRow, Row, Sqlite, SqlitePool};

use crate::config::Config;

static MIGRATIONS: [&str; 12] = [
    "CREATE TABLE event_log (id INTEGER PRIMARY KEY, created_at, type, data);",
    "CREATE TABLE index_height (blockheight INTEGER PRIMARY KEY, blockhash);",
    "CREATE TABLE raw_blockchain (id INTEGER PRIMARY KEY, blockhash, txid, blocktime, blockheight, txheight, vout, data, indexed_at);",
    "CREATE TABLE blockchain_index (id INTEGER PRIMARY KEY, protocol, fingerprint, nsid, name, pubkey, blockhash, txid, blocktime, blockheight, txheight, vout, indexed_at);",
    "CREATE VIEW ordered_blockchain_vw AS
        SELECT * from blockchain_index
        ORDER BY blockheight ASC, txheight ASC, vout ASC;",
    "CREATE VIEW ranked_blockchain_vw AS
        SELECT *, row_number() OVER (PARTITION BY fingerprint) as rank
        FROM ordered_blockchain_vw",
    "CREATE VIEW valid_names_vw AS
        SELECT * FROM ranked_blockchain_vw WHERE rank = 1;",
    "CREATE VIEW valid_names_records_vw AS
        SELECT vn.*, COALESCE(ne.records, '{}') as records
        FROM valid_names_vw vn
        LEFT JOIN name_events ne ON vn.nsid = ne.nsid;",
    "CREATE TABLE transfer_cache (id INTEGER PRIMARY KEY, protocol, fingerprint, nsid, name, pubkey, blockhash, txid, blocktime, blockheight, txheight, vout, indexed_at);",
    "CREATE TABLE name_events (name, fingerprint, nsid, pubkey, created_at, event_id, records, indexed_at, raw_event);",
    "CREATE UNIQUE INDEX name_events_unique_idx ON name_events(name, pubkey);",
    "CREATE INDEX name_events_created_at_idx ON name_events(created_at);",
];

pub async fn initialize(config: &Config) -> anyhow::Result<SqlitePool> {
    let conn = config.sqlite().await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS schema (version);")
        .execute(&conn)
        .await?;

    let (version,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(version) + 1, 0) FROM schema;")
            .fetch_one(&conn)
            .await?;

    for (idx, migration) in MIGRATIONS[version as usize..].iter().enumerate() {
        let version = idx as i64 + version;
        let mut tx = conn.begin().await?;
        tracing::debug!("Migrations schema version {version}");
        sqlx::query(migration).execute(&mut tx).await?;
        sqlx::query("INSERT INTO schema (version) VALUES (?);")
            .bind(version)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
    }

    Ok(conn)
}

pub struct RawBlockchain {
    pub blockhash: BlockHash,
    pub txid: Txid,
    pub blocktime: usize,
    pub blockheight: usize,
    pub txheight: usize,
    pub vout: usize,
    pub data: Vec<u8>,
}

impl FromRow<'_, SqliteRow> for RawBlockchain {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(RawBlockchain {
            blockhash: BlockHash::from_str(row.try_get("blockhash")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            txid: Txid::from_str(row.try_get("txid")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            blocktime: row.try_get::<i64, _>("blocktime")? as usize,
            blockheight: row.try_get::<i64, _>("blockheight")? as usize,
            txheight: row.try_get::<i64, _>("txheight")? as usize,
            vout: row.try_get::<i64, _>("vout")? as usize,
            data: hex::decode(row.try_get::<String, _>("data")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }
}

pub async fn insert_raw_blockchain(
    conn: impl Executor<'_, Database = Sqlite>,
    raw: &RawBlockchain,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_raw_blockchain.sql"))
        .bind(raw.blockhash.to_string())
        .bind(raw.txid.to_string())
        .bind(raw.blocktime as i64)
        .bind(raw.blockheight as i64)
        .bind(raw.txheight as i64)
        .bind(raw.vout as i64)
        .bind(hex::encode(&raw.data))
        .execute(conn)
        .await?;
    Ok(())
}

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

pub async fn insert_index_height(
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

pub async fn update_index_for_transfer(
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

pub async fn delete_from_transfer_cache(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    id: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM transfer_cache WHERE id = ?;")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

#[derive(FromRow)]
pub struct NameDetails {
    pub blockhash: String,
    pub txid: String,
    pub blocktime: i64,
    pub vout: i64,
    pub blockheight: i64,
    pub name: String,
    pub records: String,
    pub pubkey: String,
}

pub async fn name_details(conn: &SqlitePool, query: &str) -> anyhow::Result<NameDetails> {
    let details = sqlx::query_as::<_, NameDetails>(
        "SELECT * from valid_names_records_vw vn WHERE vn.nsid = ? or vn.name = ?",
    )
    .bind(query)
    .bind(query)
    .fetch_one(conn)
    .await?;
    Ok(details)
}

pub async fn last_records_time(conn: &SqlitePool) -> anyhow::Result<u64> {
    let (t,) = sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(created_at), 0) FROM name_events;")
        .fetch_one(conn)
        .await?;
    Ok(t as u64)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_name_event(
    conn: &SqlitePool,
    name: Name,
    fingerprint: [u8; 5],
    nsid: Nsid,
    pubkey: XOnlyPublicKey,
    created_at: i64,
    event_id: EventId,
    records: String,
    raw_event: String,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_name_event.sql"))
        .bind(name.to_string())
        .bind(hex::encode(fingerprint))
        .bind(nsid.to_string())
        .bind(pubkey.to_string())
        .bind(created_at)
        .bind(event_id.to_string())
        .bind(records)
        .bind(raw_event)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn name_records(
    conn: &SqlitePool,
    name: String,
) -> anyhow::Result<Option<HashMap<String, String>>> {
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();
    let content = sqlx::query_as::<_, (String,)>(
        "SELECT coalesce(ne.records, '{}')
        FROM valid_names_vw vn
        JOIN name_events ne ON vn.nsid = ne.nsid
        WHERE vn.fingerprint = ? LIMIT 1;",
    )
    .bind(hex::encode(fingerprint))
    .fetch_optional(conn)
    .await?;

    let records = content
        .map(|s| s.0)
        .map(|records| serde_json::from_str(&records))
        .transpose()?;
    Ok(records)
}

pub async fn top_level_names(
    conn: &SqlitePool,
    query: Option<String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let sql = match query {
        Some(q) => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw WHERE instr(name, ?) ORDER BY name;",
        )
        .bind(q.to_lowercase()),
        None => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw ORDER BY name;",
        ),
    };

    Ok(sql.fetch_all(conn).await?)
}

pub async fn save_event(conn: &SqlitePool, evt_type: &str, evt_data: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO event_log (created_at, type, data) VALUES (unixepoch(), ?, ?);")
        .bind(evt_type)
        .bind(evt_data)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn last_index_time(conn: &SqlitePool) -> anyhow::Result<i64> {
    let (created_at,) = sqlx::query_as::<_, (i64,)>(
        "SELECT created_at FROM event_log WHERE type = 'index' ORDER BY created_at DESC LIMIT 1;",
    )
    .fetch_one(conn)
    .await?;

    Ok(created_at)
}

pub type NameAndKey = (String, String);

pub async fn all_names(conn: &SqlitePool) -> anyhow::Result<Vec<NameAndKey>> {
    let rows = sqlx::query_as::<_, NameAndKey>("SELECT name, pubkey FROM valid_names_vw;")
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub enum UpgradeStatus {
    Upgraded,
    NotUpgraded,
}

pub async fn upgrade_v0_to_v1(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
    pubkey: XOnlyPublicKey,
) -> anyhow::Result<UpgradeStatus> {
    let fingerprint = hex::encode(
        Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint(),
    );
    let nsid = hex::encode(NsidBuilder::new(name, &pubkey).finalize().as_ref());

    let updated = sqlx::query(
        "UPDATE blockchain_index SET name = ?, pubkey = ?, protocol = 1 WHERE fingerprint = ? AND nsid = ? AND protocol = 0;",
    )
    .bind(name)
    .bind(hex::encode(pubkey.serialize()))
    .bind(&fingerprint)
    .bind(&nsid)
    .execute(conn)
    .await?;

    if updated.rows_affected() > 0 {
        return Ok(UpgradeStatus::Upgraded);
    }

    Ok(UpgradeStatus::NotUpgraded)
}

pub mod stats {
    use sqlx::SqlitePool;

    pub async fn known_names(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM valid_names_vw;")
            .fetch_one(conn)
            .await?;
        Ok(count)
    }

    pub async fn index_height(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>("SELECT max(blockheight) FROM index_height;")
            .fetch_one(conn)
            .await?;
        Ok(count)
    }

    pub async fn nostr_events(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "
            WITH events as (
                SELECT count(*) as count FROM name_events
            )
            SELECT SUM(count) FROM events;
        ",
        )
        .fetch_one(conn)
        .await?;
        Ok(count)
    }
}
