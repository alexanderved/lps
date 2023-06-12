use crate::*;

use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A [`Subscription`] with and erased message type.
pub trait ErasedSubscription: sealed::Sealed {
    /// Returns a message broker which sends messages to this subscription.
    fn message_broker(&self) -> Option<Arc<dyn MessageBroker>>;

    /// Returns if the subscription is registered.
    fn is_registered(&self) -> bool;
    /// Returns if the subscription is active.
    fn is_active(&self) -> bool;

    /// Registers the subscription in the given message broker.
    fn register(&mut self, msg_broker: Arc<dyn MessageBroker>) -> Result<(), SubscriptionError>;
    /// Unregisters the subscription in the given message broker.
    fn unregister(&mut self) -> Result<(), SubscriptionError>;

    /// Activates the subscription if it was deactivated before.
    fn activate(&self) -> Result<(), SubscriptionError>;
    /// Dectivates the subscription, in other words temporary makes it stop receiving messages.
    fn deactivate(&self) -> Result<(), SubscriptionError>;

    /// Receives one message if there is any.
    fn recv_message(&self) -> Option<Arc<dyn Message>>;
    /// Returns an iterator that will attempt to yield all pending messages.
    fn message_iter(&self) -> MessageIter<'_>;
    /// Processes all pending messages by calling the given function on each one.
    fn process_messages<'f>(&self, f: Box<dyn ErasedMessageHandler + 'f>);
}

/// A type which is used for receiving messages of a specific type from the message broker.
pub struct Subscription<M: Message> {
    msg_broker: Option<Arc<dyn MessageBroker>>,
    msg_recv: Option<channel::MessageReceiver>,
    _msg_type: PhantomData<M>,
}

impl<M: Message> Subscription<M> {
    /// Creates a new [`Subscription`] which is not registered in any message broker
    /// and therefore can't be used for receiving messages.
    pub fn unregistered() -> Self {
        Self {
            msg_broker: None,
            msg_recv: None,
            _msg_type: PhantomData,
        }
    }

    /// Creates a new [`Subscription`] which is registered in the given message broker.
    pub fn new(msg_broker: Arc<dyn MessageBroker>) -> Self {
        let mut sub = Self::unregistered();
        let _ = sub.register(msg_broker);

        sub
    }

    /// Receives one message if there is any.
    pub fn recv_message(&self) -> Option<Arc<M>> {
        ErasedSubscription::recv_message(self).map(|msg| msg.as_any_arc().downcast().unwrap())
    }

    /// Processes all pending messages by calling the given function on each one.
    pub fn process_messages<F: FnMut(Arc<M>)>(&self, f: F) {
        ErasedSubscription::process_messages(self, Box::new(f.into_message_handler()));
    }
}

impl<M: Message> ErasedSubscription for Subscription<M> {
    fn message_broker(&self) -> Option<Arc<dyn MessageBroker>> {
        self.msg_broker.clone()
    }

    fn is_registered(&self) -> bool {
        self.msg_broker.is_some() && self.msg_recv.is_some()
    }

    fn is_active(&self) -> bool {
        self.msg_recv
            .as_ref()
            .is_some_and(|msg_recv| msg_recv.is_active())
    }

    fn register(&mut self, msg_broker: Arc<dyn MessageBroker>) -> Result<(), SubscriptionError> {
        if self.is_registered() {
            return Err(SubscriptionError::AlreadyRegistered);
        }

        self.msg_recv = Some(msg_broker.create_message_channel::<M>());
        self.msg_broker = Some(msg_broker);

        Ok(())
    }

    fn unregister(&mut self) -> Result<(), SubscriptionError> {
        let Some(msg_broker) = self.msg_broker.take() else {
            return Err(SubscriptionError::NotRegistered);
        };
        let Some(msg_recv) = self.msg_recv.take() else {
            return Err(SubscriptionError::NotRegistered);
        };

        msg_broker.destroy_message_channel(msg_recv);

        Ok(())
    }

    fn activate(&self) -> Result<(), SubscriptionError> {
        let Some(ref msg_recv) = self.msg_recv else {
            return Err(SubscriptionError::NotRegistered);
        };
        msg_recv.set_active(true);

        Ok(())
    }

    fn deactivate(&self) -> Result<(), SubscriptionError> {
        let Some(ref msg_recv) = self.msg_recv else {
            return Err(SubscriptionError::NotRegistered);
        };
        msg_recv.set_active(false);

        Ok(())
    }

    fn recv_message(&self) -> Option<Arc<dyn Message>> {
        let Some(ref msg_recv) = self.msg_recv else { return None };

        msg_recv.recv()
    }

    fn message_iter(&self) -> MessageIter<'_> {
        MessageIter { sub: self }
    }

    fn process_messages<'f>(&self, mut f: Box<dyn ErasedMessageHandler + 'f>) {
        while let Some(msg) = self.recv_message() {
            let _ = f.call(msg);
        }
    }
}

impl<M: Message> Default for Subscription<M> {
    fn default() -> Self {
        Self::unregistered()
    }
}

impl<M: Message> Drop for Subscription<M> {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}

pub struct MultiSubscription {
    msg_broker: Option<Arc<dyn MessageBroker>>,
    is_active: AtomicBool,
    subs: Vec<Box<dyn ErasedSubscription>>,
}

impl MultiSubscription {
    pub fn unregistered() -> Self {
        Self {
            msg_broker: None,
            is_active: AtomicBool::new(true),
            subs: Vec::new(),
        }
    }

    pub fn add<M: Message>(&mut self) -> &mut Self {
        let new_sub: Subscription<M> = if let Some(msg_broker) = self.msg_broker.clone() {
            Subscription::new(msg_broker)
        } else {
            Subscription::unregistered()
        };
        self.subs.push(Box::new(new_sub));

        self
    }
}

impl ErasedSubscription for MultiSubscription {
    fn message_broker(&self) -> Option<Arc<dyn MessageBroker>> {
        self.msg_broker.clone()
    }

    fn is_registered(&self) -> bool {
        self.msg_broker.is_some()
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    fn register(&mut self, msg_broker: Arc<dyn MessageBroker>) -> Result<(), SubscriptionError> {
        self.subs
            .iter_mut()
            .try_for_each(|sub| sub.register(Arc::clone(&msg_broker)))
    }

    fn unregister(&mut self) -> Result<(), SubscriptionError> {
        self.subs.iter_mut().try_for_each(|sub| sub.unregister())
    }

    fn activate(&self) -> Result<(), SubscriptionError> {
        self.is_active.store(true, Ordering::SeqCst);
        self.subs.iter().try_for_each(|sub| sub.activate())
    }

    fn deactivate(&self) -> Result<(), SubscriptionError> {
        self.is_active.store(false, Ordering::SeqCst);
        self.subs.iter().try_for_each(|sub| sub.deactivate())
    }

    fn recv_message(&self) -> Option<Arc<dyn Message>> {
        for sub in &self.subs {
            if let Some(msg) = sub.recv_message() {
                return Some(msg);
            }
        }

        None
    }

    fn message_iter(&self) -> MessageIter<'_> {
        MessageIter { sub: self }
    }

    fn process_messages<'f>(&self, mut f: Box<dyn ErasedMessageHandler + 'f>) {
        while let Some(msg) = self.recv_message() {
            let _ = f.call(msg);
        }
    }
}

#[derive(Debug)]
pub enum SubscriptionError {
    AlreadyRegistered,
    NotRegistered,
}

mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}

    impl<M: crate::Message> Sealed for crate::Subscription<M> {}
    impl Sealed for crate::MultiSubscription {}
}
