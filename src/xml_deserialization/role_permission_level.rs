extern crate serde;
extern crate quick_xml;

#[derive(Debug, Clone, Serialize, Deserialize)]
crate struct RolePermissionLevel<K, V> {
    #[serde(rename = "RoleId")]
    crate role_id: K,

    #[serde(rename = "PermissionInteger")]
    crate permission_integer: V,
}

