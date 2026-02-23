use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("record not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("foreign key violation: {0}")]
    ForeignKey(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error(transparent)]
    SeaOrm(#[from] DbErr),
}

impl DbError {
    pub fn from_sea(err: DbErr) -> Self {
        let msg = err.to_string();

        if msg.contains("23505") {
            return DbError::Conflict(msg);
        }

        if msg.contains("23503") {
            return DbError::ForeignKey(msg);
        }

        DbError::SeaOrm(err)
    }
}

pub type DbResult<T> = Result<T, DbError>;