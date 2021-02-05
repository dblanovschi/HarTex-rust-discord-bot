use std::{
    convert::TryInto,
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

use chrono::{
    DateTime,
    Local
};

use twilight_model::{
    gateway::{
        presence::{
            Activity,
            ActivityType,
        },
    }
};

crate mod bot_configuration;
crate mod event_handler;
crate mod internal_bot_error;
crate mod model;
crate mod terminal;
crate mod twilight_http_client_extensions;
crate mod twilight_id_extensions;
crate mod whitelisting;

crate type SystemResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

crate enum EventType {
    TwilightEvent,
    CustomEvent
}

#[derive(Debug)]
crate struct SystemError(crate String);

impl Display for SystemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Error for SystemError {}

#[derive(Copy, Clone)]
crate struct Stopwatch {
    start: DateTime<Local>
}

impl Stopwatch {
    pub fn new() -> Self {
        Stopwatch {
            start: Local::now()
        }
    }

    #[allow(dead_code)]
    pub fn elapsed_seconds(&self) -> u128 {
        let now = Local::now();

        (now - self.start).num_seconds().try_into().unwrap()
    }

    pub fn elapsed_milliseconds(&self) -> u128 {
        let now = Local::now();

        (now - self.start).num_milliseconds().try_into().unwrap()
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

crate fn set_bot_activity() -> Activity {
    Activity {
        application_id: None,
        assets: None,
        created_at: None,
        details: None,
        emoji: None,
        flags: None,
        id: None,
        instance: None,
        kind: ActivityType::Watching,
        name: String::from("Being developed & stabilized"),
        party: None,
        secrets: None,
        state: None,
        timestamps: None,
        url: None
    }
}
