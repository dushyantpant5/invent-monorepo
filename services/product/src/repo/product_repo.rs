use db::Db;

pub async fn _example_query(_pool: &Db) -> anyhow::Result<()> {
    // Example: sqlx::query!("SELECT 1").execute(pool).await?;
    Ok(())
}
