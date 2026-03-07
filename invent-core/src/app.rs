use axum::{http::StatusCode, middleware as axum_middleware, routing::get, Extension, Json, Router};
use db::{check_connection, Db};
use serde_json::json;
use std::sync::Arc;

use crate::{
    domain::product::{routes as product_routes, ProductService},
    middleware::{AuthConfig, jwt_middleware},
};

/// Assembles the full Axum application.
///
/// This is the single place where:
///   - domains are registered (nest a new `domain::foo::routes(...)` here)
///   - global middleware is applied
///   - infrastructure routes (health, ready) are declared
///
/// `main.rs` is responsible only for wiring dependencies and calling this.
pub fn build_app(
    db: Db,
    product_service: Arc<dyn ProductService>,
    auth_config: AuthConfig,
) -> Router {
    // ── Authenticated API routes ───────────────────────────────────────────
    //
    // All routes nested here are protected by JWT middleware.
    // To add a new domain: nest its routes function under a new path.
    let api_router = Router::new()
        .nest("/product", product_routes::routes(product_service))
        .layer(axum_middleware::from_fn_with_state(
            auth_config,
            jwt_middleware,
        ));

    // ── Root application ───────────────────────────────────────────────────
    let shared_db = db.clone();

    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/ready",
            get({
                let db = shared_db.clone();
                move || {
                    let db = db.clone();
                    async move {
                        if check_connection(&db).await {
                            Ok(Json(json!({ "ok": true })))
                        } else {
                            Err(StatusCode::SERVICE_UNAVAILABLE)
                        }
                    }
                }
            }),
        )
        .nest("/api/v1", api_router)
        .layer(Extension(shared_db))
}
