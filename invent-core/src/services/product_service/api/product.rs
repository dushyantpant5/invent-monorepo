use crate::extractors::AuthUser;
use crate::services::product_service::model::product::Product;
use crate::services::product_service::repo::product_repo::RepoExtractor;
use axum::{extract::Json, http::StatusCode, Json as AxumJson};
use serde::Deserialize;
use uuid::Uuid;
use crate::common::ProductPermission;
#[derive(Deserialize)]
pub struct CreateProductPayload {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub quantity: i32,
}

/// POST /api/product
pub async fn create_product(
    RepoExtractor(repo): RepoExtractor,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateProductPayload>,
) -> Result<(StatusCode, AxumJson<Product>), (StatusCode, String)> {

    if !ProductPermission::can_create_product(&user){
        return Err((StatusCode::FORBIDDEN, "You are not allowed to create products".into()));
    }

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".into()));
    }
    if payload.price < 0.0 {
        return Err((StatusCode::BAD_REQUEST, "price must be >= 0".into()));
    }
    if payload.quantity < 0 {
        return Err((StatusCode::BAD_REQUEST, "quantity must be >= 0".into()));
    }

    let user_id = user.user_id;

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

            let status = match &e {
                sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                    Some("23505") => StatusCode::CONFLICT,
                    Some("23503") => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status, "database error".into()))
        }
    }
}
