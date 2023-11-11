use nomen_core::Hash160;
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
pub struct NameRecords {
    pub blockhash: String,
    pub txid: String,
    pub fingerprint: String,
    pub nsid: String,
    pub protocol: i64,
    pub records: String,
}

pub async fn name_records(conn: &SqlitePool, name: String) -> anyhow::Result<Option<NameRecords>> {
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();
    let records = sqlx::query_as::<_, NameRecords>(
        "SELECT vn.blockhash, vn.txid, vn.fingerprint, vn.nsid, vn.protocol, coalesce(ne.records, '{}') as records
        FROM valid_names_vw vn
        JOIN name_events ne ON vn.nsid = ne.nsid
        WHERE vn.fingerprint = ? LIMIT 1;",
    )
    .bind(hex::encode(fingerprint))
    .fetch_optional(conn)
    .await?;
    Ok(records)
}
