extern crate serde;
extern crate quick_xml;

use serde::{
    de::{
        Error
    },
    Deserialize,
    Deserializer
};

use super::DashboardPermissionLevel;

#[derive(Debug, Clone, Serialize)]
crate struct User {
    crate id: u64,
    crate permission_level: DashboardPermissionLevel
}

#[derive(Debug, Serialize, Deserialize)]
struct TemporaryUser {
    #[serde(rename = "Id")]
    id: u64,
    #[serde(rename = "PermissionInteger")]
    permission_integer: u8
}

impl<'deserialize> Deserialize<'deserialize> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'deserialize> {
        let deserialized_temp = <TemporaryUser as Deserialize<'deserialize>>::deserialize(deserializer)?;
        let level = match deserialized_temp.permission_integer {
            1 => DashboardPermissionLevel::Viewer,
            2 => DashboardPermissionLevel::Editor,
            3 => DashboardPermissionLevel::Admin,
            _ => return Err(D::Error::custom("Invalid permission level; expected 1, 2 or 3."))
        };

        Ok(
            Self {
                id: deserialized_temp.id,
                permission_level: level
            }
        )
    }
}
