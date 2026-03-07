use crate::shared::AppError;
use db::entities::Product;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub quantity: i32,
}

impl CreateProductRequest {
    /// Validates the request before it reaches the service layer.
    /// Returns `AppError::BadRequest` on failure so the handler can reject
    /// early without ever calling the database.
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequest("name is required".into()));
        }
        if self.price < Decimal::ZERO {
            return Err(AppError::BadRequest("price must be >= 0".into()));
        }
        if self.quantity < 0 {
            return Err(AppError::BadRequest("quantity must be >= 0".into()));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
}

impl UpdateProductRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(ref n) = self.name {
            if n.trim().is_empty() {
                return Err(AppError::BadRequest("name cannot be empty".into()));
            }
        }
        if let Some(p) = self.price {
            if p < Decimal::ZERO {
                return Err(AppError::BadRequest("price must be >= 0".into()));
            }
        }
        if let Some(q) = self.quantity {
            if q < 0 {
                return Err(AppError::BadRequest("quantity must be >= 0".into()));
            }
        }
        Ok(())
    }
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub user_id: Uuid,
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
            user_id: p.user_id,
            name: p.name,
            description: p.description,
            price: p.price,
            quantity: p.quantity,
            category_id: p.category_id,
        }
    }
}
