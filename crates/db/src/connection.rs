use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};

pub type Db = DatabaseConnection;

pub async fn get_db(database_url: &str) -> Result<Db, DbErr> {
    let db = Database::connect(database_url).await?;
    tracing::info!("database connected");
    Ok(db)
}

pub async fn check_connection(db: &Db) -> bool {
    db.execute_unprepared("SELECT 1").await.is_ok()
}
