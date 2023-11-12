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
