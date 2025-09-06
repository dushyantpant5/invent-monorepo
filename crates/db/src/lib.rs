use sqlx::{postgres::PgPoolOptions, PgPool};

pub type Db = PgPool;

pub async fn get_pool(database_url: &str, max_connections: u32) -> anyhow::Result<Db> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn check_connection(pool: &PgPool) -> bool {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("DB check_connection failed: {:?}", e);
            false
        }
    }
}
