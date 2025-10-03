use axum::{routing::post, Router};
pub mod product;

/// Register product routes
pub fn routes() -> axum::Router {
    Router::new().route("/", post(product::create_product)) // POST /product
}
