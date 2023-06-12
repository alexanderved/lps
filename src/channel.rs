use crate::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

// A unique id associated with a message channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MessageChannelId(usize);

impl MessageChannelId {
    // Creates a new [`MessageChannelId`] by casting a pointer to [`MessageTypeId`]
    // which is stored in the given [`MessageSender`] to [`usize`].
    //
    // The pointer to [`MessageChannelId`] is shared between a sender and a receiver,
    // so [`MessageChannelId`]s obtained from them will be the same.
    pub(crate) fn from_sender(msg_send: &MessageSender) -> Self {
        Self(msg_send.msg_type_id.as_ref() as *const _ as usize)
    }

    // Creates a new [`MessageChannelId`] by casting a pointer to [`MessageTypeId`]
    // which is stored in the given [`MessageReceiver`] to [`usize`].
    //
    // The pointer to [`MessageChannelId`] is shared between a sender and a receiver,
    // so [`MessageChannelId`]s obtained from them will be the same.
    pub(crate) fn from_receiver(msg_recv: &MessageReceiver) -> Self {
        Self(msg_recv.msg_type_id.as_ref() as *const _ as usize)
    }
}

// The sending-half of the message channel.
pub(crate) struct MessageSender {
    msg_type_id: Arc<MessageTypeId>,
    send: Sender<Arc<dyn Message>>,
    is_active: Arc<AtomicBool>,
}

impl MessageSender {
    // Returns the id of the channel which the [`MessageSender`] is part of.
    pub(crate) fn channel_id(&self) -> MessageChannelId {
        MessageChannelId::from_sender(self)
    }

    // Returns the type id of messages which can be sent through this [`MessageSender`].
    pub(crate) fn message_type_id(&self) -> MessageTypeId {
        *self.msg_type_id
    }

    // Returns if the channel is active.
    pub(crate) fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    // Makes the channel active or not depending on `is_active`.
    #[allow(dead_code)]
    pub(crate) fn set_active(&self, is_active: bool) {
        self.is_active.store(is_active, Ordering::SeqCst);
    }

    // Sends the given messages if messages of its types are supported by the channel.
    pub(crate) fn send(&self, msg: Arc<dyn Message>) -> Result<(), MessageChannelError> {
        if msg.type_id() == self.message_type_id() {
            return self
                .send
                .send(msg)
                .map_err(|_| MessageChannelError::MessageNotSent);
        }

        Err(MessageChannelError::WrongMessageType)
    }
}

// The receiving-half of the message channel.
pub(crate) struct MessageReceiver {
    msg_type_id: Arc<MessageTypeId>,
    recv: Receiver<Arc<dyn Message>>,
    is_active: Arc<AtomicBool>,
}

impl MessageReceiver {
    // Returns the id of the channel which the [`MessageReceiver`] is part of.
    pub(crate) fn channel_id(&self) -> MessageChannelId {
        MessageChannelId::from_receiver(self)
    }

    // Returns the type id of messages which can be received through this [`MessageReceiver`].
    pub(crate) fn message_type_id(&self) -> MessageTypeId {
        *self.msg_type_id
    }

    // Returns if the channel is active.
    pub(crate) fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    // Makes the channel active or not depending on `is_active`.
    pub(crate) fn set_active(&self, is_active: bool) {
        self.is_active.store(is_active, Ordering::SeqCst);
    }

    // Receives the message if there is any.
    pub(crate) fn recv(&self) -> Option<Arc<dyn Message>> {
        self.recv.try_recv().ok()
    }
}

// Creates a new message channel which can be used for sending messages with the given type id.
pub(crate) fn message_channel_new(msg_type_id: MessageTypeId) -> (MessageSender, MessageReceiver) {
    let msg_type_id = Arc::new(msg_type_id);
    let (send, recv) = mpsc::channel();
    let is_active = Arc::new(AtomicBool::new(true));

    let msg_send = MessageSender {
        msg_type_id: Arc::clone(&msg_type_id),
        send,
        is_active: Arc::clone(&is_active),
    };
    let msg_recv = MessageReceiver {
        msg_type_id: Arc::clone(&msg_type_id),
        recv,
        is_active,
    };

    (msg_send, msg_recv)
}

#[derive(Debug)]
pub enum MessageChannelError {
    WrongMessageType,
    MessageNotSent,
}
