use axum::{extract::State, http::StatusCode, Json};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use db::entities::Product;

use crate::{
    common::ProductPermission, errors::AppError, extractors::AuthUser,
    services::product::service::ProductService,
};

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub quantity: i32,
    pub category_id: Option<Uuid>,
}

impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            price: p.price,
            quantity: p.quantity,
            category_id: p.category_id,
        }
    }
}

pub async fn create_product(
    State(service): State<Arc<ProductService>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    if !ProductPermission::can_create_product(&user) {
        return Err(AppError::Forbidden(
            "you are not allowed to create products",
        ));
    }

    let product = service
        .create_product(
            user.user_id,
            payload.category_id,
            payload.name,
            payload.description,
            payload.price,
            payload.quantity,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(product.into())))
}
