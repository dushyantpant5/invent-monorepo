use axum::Router;
use hyper::Server;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

pub async fn run_service(app: Router, addr: SocketAddr, service_name: &str) -> anyhow::Result<()> {
    info!(service = service_name, "starting service at {}", addr);

    let server = Server::bind(&addr).serve(app.into_make_service());

    let graceful = server.with_graceful_shutdown(async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        info!(service = service_name, "shutdown signal received");
    });

    graceful.await?;
    info!(service = service_name, "service stopped");
    Ok(())
}
