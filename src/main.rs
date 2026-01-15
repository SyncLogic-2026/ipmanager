mod config;
mod db;
mod domain;
pub mod dhcp;
mod web;

use anyhow::{Context, Result};
use std::{net::SocketAddr, sync::Arc};
use tera::Tera;
use tower_sessions::{cookie::Key, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let cfg = config::Config::from_env()?;

    let pool = db::connect(&cfg).await?;
    db::ensure_initial_admin(&cfg, &pool).await?;

    dhcp::dnsmasq::sync_dnsmasq_hosts(&pool, &cfg)
        .await
        .expect("Initialer dnsmasq Sync fehlgeschlagen");

    let templates = Tera::new("templates/**/*").context("failed to load templates")?;
    let state = web::AppState {
        pool: pool.clone(),
        templates: Arc::new(templates),
        config: cfg.clone(),
    };

    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .context("failed to migrate session store")?;
    let session_key = Key::from(cfg.session_secret.as_bytes());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(cfg.session_cookie_name.clone())
        .with_secure(cfg.session_cookie_secure)
        .with_signed(session_key);

    let app = web::router(state).layer(session_layer);
    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr)
        .await
        .context("failed to bind server address")?;
    tracing::info!(addr = %cfg.bind_addr, "listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("server error")?;

    Ok(())
}
