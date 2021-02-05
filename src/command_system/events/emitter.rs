use super::{
    events::SystemEvent,
    listener::Listeners
};

/// Represents an event emitter.
#[derive(Clone)]
crate struct CommandEventEmitter {
    listeners: Listeners<SystemEvent>
}

impl CommandEventEmitter {
    /// Creates a new emitter with a set of listeners.
    crate fn new(listeners: Listeners<SystemEvent>) -> Self {
        Self {
            listeners
        }
    }

    /// Returns the emitters that this instance of the emitter has.
    crate fn into_listeners(self) -> Listeners<SystemEvent> {
        self.listeners
    }

    /// Creates an event for the listeners
    crate fn event(&self, event: SystemEvent) {
        let listener_count = self.listeners.len();
        let mut event = Some(event);

        self.send(
            |idx| {
                if idx == listener_count {
                    event.take().unwrap()
                }
                else {
                    event.clone().unwrap()
                }
            }
        )
    }

    /// Sends the events created from the `event` method.
    fn send(&self, mut f: impl FnMut(usize) -> SystemEvent) {
        let mut idx = 0;

        self.listeners.all().retain(|id, listener| {
            idx += 1;

            listener.tx.unbounded_send(f(idx)).is_ok()
        });
    }
}
