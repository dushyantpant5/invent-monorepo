use super::{handlers, service::ProductService};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Registers all product routes under a single sub-router.
///
/// To add a new product endpoint:
///   1. Write the handler in `handlers.rs`
///   2. Add one `.route(...)` line below
///
/// The returned `Router` is nested under `/product` in `app.rs`.
pub fn routes(service: Arc<dyn ProductService>) -> Router {
    Router::new()
        .route("/", post(handlers::create_product))
        .route("/my", get(handlers::list_my_products))
        .route("/:id", get(handlers::get_product))
        .route("/:id", put(handlers::update_product))
        .route("/:id", delete(handlers::delete_product))
        .with_state(service)
}
