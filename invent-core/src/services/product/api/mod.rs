pub mod handlers;

use axum::{routing::post, Router};
use crate::services::product::service::ProductService;
use std::sync::Arc;

pub fn routes(service: Arc<ProductService>) -> Router {
    Router::new()
        .route("/", post(handlers::create_product))
        .with_state(service)
}
