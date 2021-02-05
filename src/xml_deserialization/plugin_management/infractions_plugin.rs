extern crate serde;
extern crate quick_xml;

use super::command::infractions::{
    MuteCommand
};

#[derive(Debug, Serialize, Deserialize)]
crate struct InfractionsPlugin {
    #[serde(rename = "MuteCommand")]
    crate mute_command: MuteCommand
}
