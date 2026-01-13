use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::process::Command;

const DNSMASQ_CONF_PATH: &str = "/etc/dnsmasq.d/01-rust-managed.conf";

#[derive(Debug, FromRow)]
struct Host {
    mac_address: String,
    ip_address: IpNetwork,
    hostname: Option<String>,
}

async fn sync_dnsmasq_hosts(pool: &PgPool) -> Result<()> {
    let hosts: Vec<Host> = sqlx::query_as(
        "SELECT mac_address, ip_address, hostname FROM hosts",
    )
    .fetch_all(pool)
    .await
    .context("failed to fetch hosts from database")?;

    let mut output = String::new();
    for host in hosts {
        let hostname = host.hostname.as_deref().unwrap_or("");
        output.push_str(&format!(
            "dhcp-host={},{},{},infinite\n",
            host.mac_address,
            host.ip_address.ip(),
            hostname
        ));
    }

    tokio::fs::write(DNSMASQ_CONF_PATH, output)
        .await
        .with_context(|| format!("failed to write dnsmasq config to {DNSMASQ_CONF_PATH}"))?;

    let status = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("dnsmasq")
        .status()
        .context("failed to restart dnsmasq via systemctl")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "dnsmasq restart failed with status: {status}"
        ));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL is not set")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    sync_dnsmasq_hosts(&pool).await
}
