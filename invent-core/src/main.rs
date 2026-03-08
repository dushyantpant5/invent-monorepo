mod app;
mod domain;
mod extractors;
mod middleware;
mod shared;

use app_runner::run_service;
use db::{check_connection, get_db, repos::ProductRepository};
use domain::product::service::{ProductRepo, ProductService, ProductServiceImpl};
use middleware::AuthConfig;
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

    // ── Dependency graph ───────────────────────────────────────────────────
    //
    // Wire concrete implementations to their trait interfaces here.
    // Handlers and services never see concrete types — only trait objects.
    let product_repo: Arc<dyn ProductRepo> = Arc::new(ProductRepository::new(db.clone()));
    let product_service: Arc<dyn ProductService> = Arc::new(ProductServiceImpl::new(product_repo));

    let auth_config = AuthConfig::from_env();

    // ── Start server ───────────────────────────────────────────────────────
    let app = app::build_app(db, product_service, auth_config);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    run_service(app, addr, "invent-core").await
}
