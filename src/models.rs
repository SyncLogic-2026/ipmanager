use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "boot_action", rename_all = "UPPERCASE")]
pub enum NextBootAction {
    Local,
    Install,
    Shell,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Host {
    pub id: Uuid,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub subnet_id: Uuid,
    pub pxe_enabled: bool,
    pub os_type: Option<String>,
    pub boot_target: String,
    pub next_boot_action: Option<NextBootAction>,
}
