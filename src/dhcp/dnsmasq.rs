use anyhow::{Context, Result};
use sqlx::{FromRow, PgPool};
use tokio::process::Command;

#[derive(Debug, FromRow)]
struct HostRow {
    mac_address: String,
    ip_address: String,
    hostname: Option<String>,
}

pub async fn sync_dnsmasq_hosts(pool: &PgPool, config: &crate::config::Config) -> Result<()> {
    let hosts: Vec<HostRow> = match sqlx::query_as(
        "select mac_address::text as mac_address, host(ip_address) as ip_address, hostname from hosts",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = ?e, "failed to fetch hosts for dnsmasq sync");
            return Err(e).context("failed to fetch hosts for dnsmasq sync");
        }
    };

    let mut output = String::new();
    for host in hosts {
        let hostname = host.hostname.as_deref().unwrap_or("");
        output.push_str(&format!(
            "dhcp-host={},{},{}\n",
            host.mac_address, host.ip_address, hostname
        ));
    }

    if let Err(e) = tokio::fs::write(&config.dnsmasq_hosts_file, output).await {
        tracing::error!(
            error = ?e,
            path = %config.dnsmasq_hosts_file,
            "failed to write dnsmasq hosts file"
        );
        return Err(e).with_context(|| {
            format!(
                "failed to write dnsmasq hosts file to {}",
                config.dnsmasq_hosts_file
            )
        });
    }

    tracing::info!(
        path = %config.dnsmasq_hosts_file,
        "dnsmasq hosts file written"
    );

    let status = match Command::new("sh")
        .arg("-c")
        .arg(&config.dnsmasq_reload_cmd)
        .status()
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                error = ?e,
                command = %config.dnsmasq_reload_cmd,
                "failed to execute dnsmasq reload command"
            );
            return Err(e).context("failed to execute dnsmasq reload command");
        }
    };

    if !status.success() {
        tracing::error!(
            status = %status,
            command = %config.dnsmasq_reload_cmd,
            "dnsmasq reload command failed"
        );
        return Err(anyhow::anyhow!(
            "dnsmasq reload command failed with status: {status}"
        ));
    }

    tracing::info!(
        command = %config.dnsmasq_reload_cmd,
        "dnsmasq reload command succeeded"
    );

    Ok(())
}
