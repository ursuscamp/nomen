use crate::config::Config;

use sqlx::SqlitePool;

pub mod event_log;
pub mod index;
pub mod name;
pub mod raw;
pub mod relay_index;
pub mod stats;

static MIGRATIONS: [&str; 18] = [
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
    "CREATE TABLE relay_index_queue (name);",
    "ALTER TABLE blockchain_index ADD COLUMN v1_upgrade_blockheight;",
    "ALTER TABLE blockchain_index ADD COLUMN v1_upgrade_txid",
    "CREATE UNIQUE INDEX riq_name_idx ON relay_index_queue (name)",
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
