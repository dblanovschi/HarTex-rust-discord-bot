use twilight_model::{
    id::RoleId
};

const CONTENT_DISTRIBUTION_NETWORK_BASE_URL: &str = "https://cdn.discordapp.com/";

crate const fn content_distribution_network_base_url() -> &'static str {
    CONTENT_DISTRIBUTION_NETWORK_BASE_URL
}

crate const fn hartex_guild_owner() -> RoleId {
    RoleId(791588740270784512)
}

crate const fn verified_hartex_user() -> RoleId {
    RoleId(791588599661199410)
}

crate const fn hartex_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
