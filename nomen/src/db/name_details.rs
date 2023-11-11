use sqlx::{FromRow, SqlitePool};

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
    pub protocol: i64,
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
