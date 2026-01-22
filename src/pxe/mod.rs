use anyhow::{Context, Result};
use std::str::FromStr;
use tokio::io::AsyncWriteExt;

use crate::config::Config;
use crate::domain::mac::MacAddr;

#[derive(Debug, Clone)]
pub struct HostPxe {
    pub mac_address: String,
    pub os_type: Option<String>,
}

pub async fn ensure_ipxe_configs_dir(config: &Config) -> Result<std::path::PathBuf> {
    let configs_dir = std::path::Path::new(&config.pxe_configs_dir).to_path_buf();
    tokio::fs::create_dir_all(&configs_dir)
        .await
        .context("failed to create pxe configs directory")?;
    Ok(configs_dir)
}

pub async fn write_ipxe_config(host: &HostPxe, configs_dir: &std::path::Path) -> Result<()> {
    let mac = MacAddr::from_str(host.mac_address.trim())
        .with_context(|| format!("invalid mac_address in hosts table: {}", host.mac_address))?;
    let mac_dash = mac.to_string().replace(':', "-");
    let script = render_ipxe_script(host.os_type.as_deref());
    let file_path = configs_dir.join(format!("host-{}.ipxe", mac_dash));
    let mut file = tokio::fs::File::create(&file_path)
        .await
        .with_context(|| format!("failed to create ipxe config {}", file_path.display()))?;
    file.write_all(script.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

fn render_ipxe_script(os_type: Option<&str>) -> String {
    match os_type.map(|v| v.trim().to_lowercase()) {
        Some(ref v) if v == "ubuntu" => {
            "#!ipxe\n\
kernel http://10.70.99.33:3000/pxe-assets/vmlinuz initrd=initrd ip=dhcp autoinstall ds=nocloud-net;s=http://10.70.99.33:3000/cloud-init/${mac}/\n\
initrd http://10.70.99.33:3000/pxe-assets/initrd\n\
boot\n"
                .to_string()
        }
        _ => "#!ipxe\nshell\n".to_string(),
    }
}
