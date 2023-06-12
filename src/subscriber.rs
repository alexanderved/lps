use crate::*;

use std::sync::Arc;

/// A receiver of messages.
///
/// [`Subscriber`] can receive multiple types of messages by storing multiple [`Subscription`]s.
pub trait Subscriber {
    /// Makes the subscriber listen for messages from the given message broker
    /// by registering all its [`Subscription`]s (see [`ErasedSubscription::register`]).
    fn subscribe(&mut self, msg_broker: Arc<dyn MessageBroker>);
    /// Makes the subscriber stop listening for messages from the given message broker
    /// by unregistering all its [`Subscription`]s (see [`ErasedSubscription::unregister`]).
    fn unsubscribe(&mut self);

    /// Makes the subscriber listen for messages by activating all its
    /// [`Subscription`]s (see [`ErasedSubscription::activate`]) if
    /// they were deactivated before.
    fn activate(&self);
    /// Temporary makes the subscriber not receive any messages
    /// by deactivating all its [`Subscription`]s (see [`ErasedSubscription::deactivate`]).
    fn deactivate(&self);

    /// Proccesses all messages received from all [`Subscription`]s of the subscriber.
    ///
    /// See [`Subscription::process_messages`]
    fn process_messages(&mut self);
}
