pub mod handlers;

use crate::services::product::service::ProductService;
use axum::{routing::post, Router};
use std::sync::Arc;

pub fn routes(service: Arc<ProductService>) -> Router {
    Router::new()
        .route("/", post(handlers::create_product))
        .with_state(service)
}
