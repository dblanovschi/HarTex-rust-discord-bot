crate struct BotConfiguration {
    pub token: String
}

impl BotConfiguration {
    crate fn new(token: String) -> Self {
        Self {
            token,
        }
    }
}
