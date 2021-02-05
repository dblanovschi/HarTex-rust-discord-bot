extern crate serde;
extern crate quick_xml;

#[derive(Debug, Serialize, Deserialize)]
crate struct MuteCommand {
    #[serde(rename = "MutedRole", default)]
    crate muted_role: Option<MutedRole>,

    #[serde(rename = "RoleToRemove", default)]
    crate role_to_remove: Option<RoleToRemove>,
}

#[derive(Debug, Serialize, Deserialize)]
crate struct MutedRole {
    #[serde(rename = "RoleId")]
    crate role_id: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
crate struct RoleToRemove {
    #[serde(rename = "RoleId")]
    crate role_id: u64
}
