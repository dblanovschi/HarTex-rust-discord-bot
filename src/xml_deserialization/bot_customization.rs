extern crate serde;
extern crate quick_xml;

#[derive(Debug, Serialize, Deserialize)]
crate struct BotCustomization {
    #[serde(rename = "CommandPrefix")]
    crate command_prefix: String,

    #[serde(rename = "GuildNickname")]
    crate guild_nickname: String,
}

impl Default for BotCustomization {
    fn default() -> Self {
        Self {
            command_prefix: String::from("hb."),
            guild_nickname: String::from("HarTex")
        }
    }
}
