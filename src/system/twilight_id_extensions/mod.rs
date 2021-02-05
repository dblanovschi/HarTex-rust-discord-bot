use twilight_model::{
    id::{
        AttachmentId,
        ApplicationId,
        AuditLogEntryId,
        ChannelId,
        EmojiId,
        GenericId,
        GuildId,
        IntegrationId,
        MessageId,
        RoleId,
        UserId,
        WebhookId
    },
    oauth::{
        id::TeamId
    }
};

crate trait IntoInnerU64 {
    fn into_inner_u64(self) -> u64;
}

impl IntoInnerU64 for AttachmentId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for ApplicationId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for AuditLogEntryId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for ChannelId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for EmojiId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for GenericId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for GuildId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for IntegrationId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for MessageId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for RoleId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for UserId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for WebhookId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}

impl IntoInnerU64 for TeamId {
    fn into_inner_u64(self) -> u64 {
        self.0
    }
}
