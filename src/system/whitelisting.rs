use twilight_model::{
    id::GuildId
};

#[derive(Clone)]
pub struct WhitelistedGuilds {
    guilds: Vec<GuildId>
}
