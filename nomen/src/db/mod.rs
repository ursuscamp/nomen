use crate::config::Config;
pub use blockchain_index::{insert_blockchain_index, insert_transfer_cache, BlockchainIndex};
pub use index::{insert_index_height, next_index_height, update_index_for_transfer};
pub use name_details::{name_details, NameDetails};
pub use name_records::{name_records, NameRecords};
use nomen_core::{Hash160, Name, Nsid, NsidBuilder};
use nostr_sdk::EventId;
pub use raw::{insert_raw_blockchain, RawBlockchain};
use secp256k1::XOnlyPublicKey;
use sqlx::{Sqlite, SqlitePool};

mod blockchain_index;
mod index;
mod name_details;
mod name_records;
mod raw;
pub mod stats;

static MIGRATIONS: [&str; 14] = [
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

    // This is useful so that we can know that this blockheight was already indexed. Even if the cache entry is deleted because it's old, we can keep it here so that
    // we can know we already looked at it.
    "CREATE TABLE old_transfer_cache (id, protocol, fingerprint, nsid, name, pubkey, blockhash, txid, blocktime, blockheight, txheight, vout, indexed_at);",

    // This view is useful as an "interesting things" view. I.e., something related to Nomen existed at this blockheight and we have already seen it.
    "CREATE VIEW index_blockheights_vw AS
        SELECT blockheight FROM blockchain_index
        UNION
        SELECT blockheight FROM transfer_cache
        UNION
        SELECT blockheight FROM old_transfer_cache;",

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

pub async fn top_level_names(
    conn: &SqlitePool,
    query: Option<String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let sql = match query {
        Some(q) => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw WHERE name IS NOT NULL AND instr(name, ?) ORDER BY name;",
        )
        .bind(q.to_lowercase()),
        None => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw WHERE name IS NOT NULL ORDER BY name;",
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

pub async fn check_name_availability(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
) -> anyhow::Result<bool> {
    let fp = hex::encode(
        Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint(),
    );
    let (a,) = sqlx::query_as::<_, (bool,)>(
        "SELECT COUNT(*) = 0 FROM valid_names_vw WHERE fingerprint = ?;",
    )
    .bind(&fp)
    .fetch_one(conn)
    .await?;
    Ok(a)
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
    Ok(())
}
