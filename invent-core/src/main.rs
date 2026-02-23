mod common;
mod errors;
mod extractors;
mod middleware;
mod services;

use app_runner::run_service;
use axum::{
    http::StatusCode, middleware as axum_middleware, routing::get, Extension, Json, Router,
};
use db::{check_connection, get_db, repos::ProductRepository};
use serde_json::json;
use services::product::{api as product_api, service::ProductService};
use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = get_db(&database_url).await?;
    if !check_connection(&db).await {
        anyhow::bail!("database not ready");
    }

    let product_repo = Arc::new(ProductRepository::new(db.clone()));
    let product_service = Arc::new(ProductService::new(product_repo));

    let auth_config = middleware::AuthConfig::from_env();

    let api_router = Router::new()
        .nest("/product", product_api::routes(product_service))
        .layer(axum_middleware::from_fn_with_state(
            auth_config.clone(),
            middleware::jwt_middleware,
        ));

    let shared_db = db.clone();

    let app = Router::new()
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
        .layer(Extension(shared_db));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    run_service(app, addr, "invent-core").await
}
