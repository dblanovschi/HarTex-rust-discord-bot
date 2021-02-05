extern crate serde;
extern crate quick_xml;

use super::InfractionsPlugin;

#[derive(Debug, Serialize, Deserialize)]
crate struct Plugins {
    #[serde(rename = "InfractionsPlugin")]
    crate infractions_plugin: InfractionsPlugin
}
