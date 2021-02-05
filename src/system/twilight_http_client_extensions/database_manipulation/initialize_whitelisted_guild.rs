use std::{
    env::*,
    future::Future,
    io::Cursor,
    pin::Pin,
    task::{
        Context,
        Poll,
    }
};

use sqlx::{
    error::{
        Result as SqlxResult
    },
    postgres::{
        PgPool,
        PgRow
    },
    Row
};

use tokio_postgres::{
    NoTls,
    connect
};

use twilight_http::{
    Client
};

use twilight_model::{
    id::GuildId
};

use quick_xml::Writer;

use crate::command_system::{
    CommandError
};

use crate::logging::logger::Logger;

use crate::xml_deserialization::{
    plugin_management::{
        command::{
            infractions::{
                MuteCommand
            }
        },
        InfractionsPlugin,
        Plugins,
    },
    BotConfig,
    BotCustomization,
    Dashboard,
    DashboardPermissionLevel,
    RolePermissionLevels,
    User
};

use super::{
    super::{
        error::ClientExtensionResult,
        Pending
    },
};

crate struct InitializeWhitelistedGuild {
    future: Option<Pending<()>>,

    guild_id: GuildId,

    http: Client
}

impl InitializeWhitelistedGuild {
    crate fn new(http: Client, guild_id: GuildId) -> Self {
        Self {
            future: None,

            guild_id,

            http
        }
    }

    fn start(&mut self) -> ClientExtensionResult<()> {
        Logger::log_debug(
            "Attempting to create connection to HarTexBetaGuildConfiguration database.".to_string());

        self.future.replace(Box::pin(request(self.guild_id, self.http.clone())));

        Ok(())
    }
}

impl Future for InitializeWhitelistedGuild {
    type Output = ClientExtensionResult<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(future) = self.as_mut().future.as_mut() {
                return future.as_mut().poll(cx);
            }

            if let Err(error) = self.start() {
                return Poll::Ready(Err(error));
            }
        }
    }
}

unsafe impl Send for InitializeWhitelistedGuild {}

async fn request(guild_id: GuildId, http: Client) -> ClientExtensionResult<()> {
    let database_credentials = if let Ok(credentials) = var("PGSQL_CREDENTIALS_GUILD_CONFIGURATION") {
        credentials
    }
    else {
        return Err(box CommandError("Credentials is none.".to_string()))
    };

    let connection = PgPool::connect(
        &database_credentials
    ).await?;

    let guild_request = http.guild(guild_id).await?;

    if let Some(guild) = guild_request {
        let mut writer =
            Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 4);

        quick_xml::se::to_writer(writer.inner().get_mut(), &BotConfig {
            dashboard: Dashboard {
                users: vec![User {
                    id: guild.owner_id.0,
                    permission_level: DashboardPermissionLevel::Admin
                }]
            },
            bot_customization: BotCustomization::default(),
            role_permission_levels: RolePermissionLevels::default(),
            plugins: Plugins {
                infractions_plugin: InfractionsPlugin {
                    mute_command: MuteCommand {
                        muted_role: None,
                        role_to_remove: None
                    }
                }
            }
        })?;

        let result = writer.into_inner().into_inner();
        dbg!(String::from_utf8(result)?);

        // Assuming when this is executed, all pre-execution checks are done: the schema and tables for the guild does NOT exist.

        Ok(())
    }
    else {
        Err(box CommandError("Guild ID is invalid.".to_string()))
    }
}
