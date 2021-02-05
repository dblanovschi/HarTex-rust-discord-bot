use std::{
    convert::TryInto,
    env::*,
    future::Future,
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

use twilight_model::{
    id::GuildId
};

use crate::command_system::CommandError;
use crate::logging::logger::Logger;
use super::{
    super::{
        error::ClientExtensionResult,
        Pending
    },
};

crate struct GetWhitelistedGuilds {
    future: Option<Pending<Vec<GuildId>>>,
}

impl GetWhitelistedGuilds {
    crate fn new() -> GetWhitelistedGuilds {
        GetWhitelistedGuilds {
            future: None,
        }
    }

    fn start(&mut self) -> ClientExtensionResult<()> {
        Logger::log_debug("Attempting to create connection to HarTexBetaGuilds database.".to_string());

        self.future.replace(Box::pin(request()));

        Ok(())
    }
}

impl Default for GetWhitelistedGuilds {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for GetWhitelistedGuilds {
    type Output = ClientExtensionResult<Vec<GuildId>>;

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

unsafe impl Send for GetWhitelistedGuilds {}

async fn request() -> ClientExtensionResult<Vec<GuildId>> {
    let database_credentials = if let Ok(credentials) = var("PGSQL_CREDENTIALS_GUILDS") {
        credentials
    }
    else {
        return Err(box CommandError("Credentials is none.".to_string()))
    };

    let connection = PgPool::connect(
            &database_credentials
        ).await?;
    let query_reult: SqlxResult<Vec<PgRow>> = sqlx::query("SELECT * FROM guilds; --").fetch_all(&connection).await;

    let mut guild_id_vector = Vec::new();

    match query_reult {
        Ok(rows) => {
            Logger::log_verbose("Successfully made query.".to_string());

            for row in rows {
                let value: i64 = row.get("guild_id");

                guild_id_vector.push(GuildId(value.try_into()?));
            }
        },
        Err(error) => {
            Logger::log_error(
                format!("Failed to get list of whitelisted guilds from database. Error: {}", error.as_database_error().unwrap()));
        }
    }

    Ok(guild_id_vector)
}
