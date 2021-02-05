extern crate base64;

use std::{
    future::Future,
    env::*,
    pin::Pin,
    task::{
        Context,
        Poll,
    }
};

use base64::decode;

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
    id::{
        GuildId
    }
};

use crate::command_system::CommandError;
use crate::logging::logger::Logger;
use crate::system::{
    twilight_http_client_extensions::{
        error::ClientExtensionResult,
        Pending
    },
    twilight_id_extensions::IntoInnerU64
};

crate struct GetGuildConfiguration {
    future: Option<Pending<String>>,

    guild_id: GuildId,
}

impl GetGuildConfiguration {
    crate fn new(guild_id: GuildId) -> GetGuildConfiguration {
        GetGuildConfiguration {
            future: None,

            guild_id,
        }
    }

    fn start(&mut self) -> ClientExtensionResult<()> {
        Logger::log_debug("Attempting to create connection to HarTexBetaGuildConfiguration database."
            .to_string());

        self.future.replace(Box::pin(request(self.guild_id)));

        Ok(())
    }
}

impl Future for GetGuildConfiguration {
    type Output = ClientExtensionResult<String>;

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

unsafe impl Send for GetGuildConfiguration {}

async fn request(guild_id: GuildId) -> ClientExtensionResult<String> {
    let database_credentials = if let Ok(credentials) = var("PGSQL_CREDENTIALS_GUILD_CONFIGURATION") {
        credentials
    }
    else {
        return Err(box CommandError("Credentials is none.".to_string()))
    };

    let connection = PgPool::connect(
        &database_credentials
    ).await?;

    Logger::log_debug(
        format!("Making query to database. [Getting guild configuration for guild: {}]", guild_id
        )
    );

    let query_result = sqlx::query(
        // language=SQL
        &format!("SELECT * FROM guild_{}.plain_config; --", guild_id.into_inner_u64())
    )
        .fetch_all(&connection)
        .await;

    match query_result {
        Ok(rows) => {
            return if let Some(row) = rows.first() {
                let config: String = row.get("config");

                Ok(
                    String::from_utf8(decode(config)?)?
                )
            }
            else {
                Err(box CommandError("Configuration not found".to_string()))
            }
        }
        Err(error) => {
            Err(box error)
        }
    }
}
