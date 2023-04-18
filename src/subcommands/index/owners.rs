use sqlx::SqlitePool;

pub async fn reindex(conn: &SqlitePool) -> anyhow::Result<()> {
    let mut tx = conn.begin().await?;
    sqlx::query("DELETE FROM name_owners;")
        .execute(&mut tx)
        .await?;
    sqlx::query("INSERT INTO name_owners SELECT name, pubkey, created_at FROM name_vw;")
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
