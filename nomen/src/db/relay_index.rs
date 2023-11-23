use sqlx::Sqlite;

pub async fn queue(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
) -> anyhow::Result<()> {
    sqlx::query("INSERT OR IGNORE INTO relay_index_queue (name) VALUES (?)")
        .bind(name)
        .execute(conn)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow, Debug)]
pub struct Name {
    pub name: String,
    pub pubkey: String,
    pub records: String,
}

pub async fn fetch_all_queued(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
) -> anyhow::Result<Vec<Name>> {
    let results = sqlx::query_as::<_, Name>(
        "SELECT vnr.name, vnr.pubkey, COALESCE(vnr.records, '{}') as records
        FROM valid_names_records_vw vnr
        JOIN relay_index_queue riq ON vnr.name = riq.name;",
    )
    .fetch_all(conn)
    .await?;
    Ok(results)
}

pub async fn fetch_all(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
) -> anyhow::Result<Vec<Name>> {
    let results = sqlx::query_as::<_, Name>(
        "SELECT vnr.name, vnr.pubkey, COALESCE(vnr.records, '{}') as records
        FROM valid_names_records_vw vnr;",
    )
    .fetch_all(conn)
    .await?;
    Ok(results)
}

pub async fn delete(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM relay_index_queue WHERE name = ?;")
        .bind(name)
        .execute(conn)
        .await?;
    Ok(())
}
