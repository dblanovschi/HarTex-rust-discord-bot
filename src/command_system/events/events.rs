use std::{
    pin::Pin,
    task::{
        Context,
        Poll
    }
};

use futures_channel::mpsc::UnboundedReceiver;
use futures_util::{
    stream::Stream,
    StreamExt
};

use crate::system::model::payload::{
    CommandExecuted,
    CommandFailed,
    CommandReceived
};

/// Represents some events.
crate struct CommandEvents {
    rx: UnboundedReceiver<SystemEvent>
}

impl<'a> CommandEvents {
    crate fn new(rx: UnboundedReceiver<SystemEvent>) -> Self {
        Self {
            rx
        }
    }
}

impl Stream for CommandEvents {
    type Item = SystemEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

/// The variants of events that can be emitted.
#[allow(clippy::enum_variant_names)]
#[derive(Clone)]
crate enum SystemEvent {
    CommandReceived(Box<CommandReceived>),
    CommandIdentified(String),
    CommandExecuted(Box<CommandExecuted>),
    CommandFailed(Box<CommandFailed>),
}
