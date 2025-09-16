use axum::{extract::Json, http::StatusCode, Json as AxumJson};
use serde::Deserialize;
use uuid::Uuid;

use crate::services::product_service::model::product::Product;
use crate::services::product_service::repo::product_repo::RepoExtractor;

#[derive(Deserialize)]
pub struct CreateProductPayload {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub quantity: i32,
}

/// TODO: replace with real auth extraction. For now a stub returns a test user id.
async fn get_user_id_from_request() -> Uuid {
    Uuid::parse_str("0ef86e4f-18fa-4639-9863-d3233e20e65f").unwrap()
}
/// POST /api/product
pub async fn create_product(
    RepoExtractor(repo): RepoExtractor,
    Json(payload): Json<CreateProductPayload>,
) -> Result<(StatusCode, AxumJson<Product>), (StatusCode, String)> {
    // Basic validation
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".into()));
    }
    if payload.price < 0.0 {
        return Err((StatusCode::BAD_REQUEST, "price must be >= 0".into()));
    }
    if payload.quantity < 0 {
        return Err((StatusCode::BAD_REQUEST, "quantity must be >= 0".into()));
    }

    let user_id = get_user_id_from_request().await;

    // If you want to ensure category belongs to user, add a check here (optional).
    // For example:
    // if let Some(cat_id) = payload.category_id {
    //     if !repo.category_belongs_to_user(cat_id, user_id).await? { return Err((StatusCode::BAD_REQUEST, "invalid category".into())); }
    // }

    match repo
        .create_product(
            user_id,
            payload.category_id,
            &payload.name,
            payload.description.as_deref(),
            payload.price,
            payload.quantity,
        )
        .await
    {
        Ok(product) => Ok((StatusCode::CREATED, AxumJson(product))),
        Err(e) => {
            tracing::error!("db error creating product: {:?}", e);

            // Map a few common Postgres error codes to HTTP statuses:
            let status = match &e {
                sqlx::Error::Database(db_err) => {
                    match db_err.code().as_deref() {
                        Some("23505") => StatusCode::CONFLICT,    // unique_violation
                        Some("23503") => StatusCode::BAD_REQUEST, // foreign_key_violation
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    }
                }
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status, "database error".into()))
        }
    }
}
