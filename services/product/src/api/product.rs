use crate::model::product::Product;
use axum::http::StatusCode;
use axum::{extract::Path, Json};
use uuid::Uuid;

/// Register product routes
pub fn routes() -> axum::Router {
    axum::Router::new().route("/:id", axum::routing::get(get_product_by_id))
}

/// Get product by id
async fn get_product_by_id(Path(id): Path<Uuid>) -> Result<Json<Product>, StatusCode> {
    // TODO: replace with actual DB lookup
    let found = Some(Product {
        id,
        sku: "SKU123".to_string(),
        name: "Sample".to_string(),
        quantity: 5,
    });

    if let Some(p) = found {
        Ok(Json(p))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
