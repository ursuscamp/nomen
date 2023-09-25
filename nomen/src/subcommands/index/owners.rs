use sqlx::SqlitePool;

pub async fn reindex(conn: &SqlitePool) -> anyhow::Result<()> {
    log::info!("Beginning owner index rebuild.");
    let mut tx = conn.begin().await?;
    sqlx::query("DELETE FROM name_owners;")
        .execute(&mut tx)
        .await?;
    sqlx::query("INSERT INTO name_owners SELECT name, pubkey FROM owners_vw;")
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    log::info!("Owner index build complete.");
    Ok(())
}
