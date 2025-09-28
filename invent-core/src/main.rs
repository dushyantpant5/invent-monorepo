use app_runner::run_service;
use axum::http::StatusCode;
use axum::{routing::get, Extension, Json, Router};
use db::{check_connection, get_pool, Db};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};

mod services {
    pub mod product_service;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_conns: u32 = std::env::var("DB_MAX_CONNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let pool: Db = get_pool(&database_url, max_conns).await?;
    if !check_connection(&pool).await {
        anyhow::bail!("database not ready");
    }

    let shared_pool = Arc::new(pool);

    let api_router = Router::new().nest("/product", services::product_service::api::routes());

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/ready",
            get({
                let pool = shared_pool.clone();
                move || {
                    let pool = pool.clone();
                    async move {
                        if check_connection(&pool).await {
                            Ok(Json(json!({ "ok": true })))
                        } else {
                            Err(StatusCode::SERVICE_UNAVAILABLE)
                        }
                    }
                }
            }),
        )
        .nest("/api/v1", api_router)
        .layer(Extension(shared_pool));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    run_service(app, addr, "invent-core").await
}
