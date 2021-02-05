extern crate serde;
extern crate quick_xml;

use super::{
    plugin_management::{
        Plugins
    },
    BotCustomization,
    Dashboard,
    RolePermissionLevels
};

#[derive(Debug, Serialize, Deserialize)]
crate struct BotConfig {
    #[serde(rename = "Dashboard")]
    crate dashboard: Dashboard,

    #[serde(rename = "BotCustomization", default)]
    crate bot_customization: BotCustomization,

    #[serde(rename = "RolePermissionLevels", default)]
    crate role_permission_levels: RolePermissionLevels<u64, u32>,

    #[serde(rename = "Plugins")]
    crate plugins: Plugins
}
