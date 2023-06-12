use crate::{channel::*, *};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MessageTopic {
    msg_type_id: MessageTypeId,
    msg_senders: Mutex<HashMap<MessageChannelId, MessageSender>>,
}

impl MessageTopic {
    pub fn new(msg_type_id: MessageTypeId) -> Self {
        Self {
            msg_type_id,
            msg_senders: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_typed<M: Message>() -> Self {
        Self::new(MessageTypeId::of::<M>())
    }

    pub(crate) fn create_message_channel(&self) -> MessageReceiver {
        let (msg_send, msg_recv) = message_channel_new(self.msg_type_id);
        let mut msg_senders = self
            .msg_senders
            .lock()
            .expect("The message topic is poisoned");
        msg_senders.insert(msg_send.channel_id(), msg_send);

        msg_recv
    }

    pub(crate) fn destroy_message_channel(
        &self,
        msg_recv: MessageReceiver,
    ) -> Result<(), MessageTopicError> {
        let mut msg_senders = self
            .msg_senders
            .lock()
            .expect("The message topic is poisoned");
        msg_senders
            .remove(&msg_recv.channel_id())
            .map(|_| ())
            .ok_or(MessageTopicError::ChannelNotFound)
    }

    pub(crate) fn send_message(&self, msg: Arc<dyn Message>) -> Result<(), MessageTopicError> {
        let msg_senders = self
            .msg_senders
            .lock()
            .expect("The message topic is poisoned");
        msg_senders
            .values()
            .filter(|msg_send| msg_send.is_active())
            .try_for_each(|msg_send| msg_send.send(Arc::clone(&msg)))
            .map_err(|msg_channel_err| MessageTopicError::MessageChannelError(msg_channel_err))
    }
}

pub enum MessageTopicError {
    MessageChannelError(MessageChannelError),
    WrongMessageType,
    ChannelNotFound,
}
