extern crate serde;
extern crate quick_xml;

use super::User;

#[derive(Debug, Serialize, Deserialize)]
crate struct Dashboard {
    #[serde(rename = "DashboardUser")]
    crate users: Vec<User>
}
