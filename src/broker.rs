use crate::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[doc(hidden)]
pub trait AsMessageBroker {
    #[doc(hidden)]
    fn as_message_broker(self: Arc<Self>) -> Arc<dyn MessageBroker>;
}

impl<B: MessageBroker + 'static> AsMessageBroker for B {
    fn as_message_broker(self: Arc<Self>) -> Arc<dyn MessageBroker> {
        self
    }
}

/// A type which delivers messages from publishers to subscribers.
pub trait MessageBroker: AsMessageBroker {
    /// Gets [`MessageTopic`] which is responsible for handling messages of the given type.
    fn get_message_topic(&self, msg_type_id: MessageTypeId) -> Arc<MessageTopic>;

    /// Registers the subscription in the broker.
    ///
    /// After that the subscription can receive messages.
    fn register_subscription(
        self: Arc<Self>,
        sub: &mut dyn ErasedSubscription,
    ) -> Result<(), MessageBrokerError> {
        sub.register(self.as_message_broker())
            .map_err(|sub_err| MessageBrokerError::SubscriptionError(sub_err))
    }

    /// Unregisters the subscription in the broker.
    ///
    /// After that the subscription stops receiving messages.
    fn unregister_subscription(
        &self,
        sub: &mut dyn ErasedSubscription,
    ) -> Result<(), MessageBrokerError> {
        sub.unregister()
            .map_err(|sub_err| MessageBrokerError::SubscriptionError(sub_err))
    }

    /// Sends the given message to all subscribers which are listening for messages of its type.
    fn publish_message(&self, msg: Arc<dyn Message>) -> Result<(), MessageBrokerError> {
        self.get_message_topic(msg.type_id())
            .send_message(msg)
            .map_err(|msg_topic_err| MessageBrokerError::MessageTopicError(msg_topic_err))
    }
}

impl dyn MessageBroker {
    // Creates a new message channel in the message topic of the given generic type.
    pub(crate) fn create_message_channel<M: Message>(&self) -> channel::MessageReceiver {
        let msg_topic = self.get_message_topic(MessageTypeId::of::<M>());
        msg_topic.create_message_channel()
    }

    // Destroys the given message channel.
    pub(crate) fn destroy_message_channel(&self, msg_recv: channel::MessageReceiver) {
        let msg_topic = self.get_message_topic(msg_recv.message_type_id());
        let _ = msg_topic.destroy_message_channel(msg_recv);
    }
}

pub struct DefaultMessageBroker {
    msg_topics_map: Mutex<HashMap<MessageTypeId, Arc<MessageTopic>>>,
}

impl DefaultMessageBroker {
    pub fn new() -> Self {
        Self {
            msg_topics_map: Mutex::new(HashMap::new()),
        }
    }
}

impl MessageBroker for DefaultMessageBroker {
    fn get_message_topic(&self, msg_type_id: MessageTypeId) -> Arc<MessageTopic> {
        let mut msg_topics_map = self
            .msg_topics_map
            .lock()
            .expect("The default message broker is poisoned");
        msg_topics_map
            .entry(msg_type_id)
            .or_insert(Arc::new(MessageTopic::new(msg_type_id)));

        Arc::clone(msg_topics_map.get(&msg_type_id).unwrap())
    }
}

pub enum MessageBrokerError {
    SubscriptionError(SubscriptionError),
    MessageTopicError(MessageTopicError),
    Other(Box<dyn std::any::Any>),
}
