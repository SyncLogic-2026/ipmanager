mod config;
pub mod dhcp;

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let cfg = config::Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .connect(&cfg.database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    if let Err(e) = dhcp::dnsmasq::sync_dnsmasq_hosts(&pool, &cfg).await {
        tracing::error!(error = ?e, "initial dnsmasq sync failed");
    }

    Ok(())
}
