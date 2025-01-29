use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
};

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use tracing::info;

struct AppConfig {
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let cfg = AppConfig::new().context("failed to parse app configuration")?;

    let api_routes = Router::new().route("/health", get(health));

    let app = Router::new().nest("/api", api_routes);

    let sock_addr = SocketAddr::from((IpAddr::V6(Ipv6Addr::LOCALHOST), cfg.port));
    let listener = tokio::net::TcpListener::bind(sock_addr)
        .await
        .with_context(|| format!("failed to bind to address {sock_addr}"))?;
    info!("listening on http://{}", sock_addr);
    axum::serve(listener, app)
        .await
        .context("failed to serve web server")?;

    Ok(())
}

impl AppConfig {
    fn new() -> Result<Self> {
        let port = match env::var("PORT") {
            Ok(port) => port.parse().with_context(|| {
                format!("failed to parse port '{port}', it must be a number between 0 and 65525")
            })?,
            Err(_) => 3000,
        };
        Ok(Self { port })
    }
}

async fn health() -> &'static str {
    "OK"
}
