use crate::*;

use std::sync::Arc;

/// A sender of messages.
pub trait Publisher {
    /// Returns the message broker which the publisher connected to.
    fn message_broker(&self) -> Arc<dyn MessageBroker>;
    /// Connects the publisher to the given message broker.
    fn set_message_broker(&mut self, msg_broker: Arc<dyn MessageBroker>);

    /// Sends the given message to the message broker and lets the broker deliver it.
    fn publish(&self, msg: Arc<dyn Message>) {
        let _ = self.message_broker().publish_message(msg);
    }
}
