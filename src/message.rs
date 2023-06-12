use std::any::TypeId;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::*;

/// A type which is used for communicating between publishers and subscribers.
pub trait Message: util::AsAny + Send + Sync + 'static {
    /// Returns the type id of the message.
    fn type_id(&self) -> MessageTypeId {
        MessageTypeId(self.as_any_ref().type_id())
    }
}

/// The type id of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageTypeId(pub TypeId);

impl MessageTypeId {
    /// Gets the [`MessageTypeId`] of the given generic type.
    pub fn of<M: Message>() -> Self {
        Self(TypeId::of::<M>())
    }
}

/// A function with erased type for handling messages.
pub trait ErasedMessageHandler {
    /// Runs the function with the given message.
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessageHandlerError>;
}

/// A function for handling messages of specific type.
///
/// You can get it by calling [`IntoMessageHandler::into_message_handler`] on a function
/// which accepts a message.
pub struct MessageHandler<M: ?Sized, F> {
    f: F,
    _marker: PhantomData<fn(Arc<M>)>,
}

impl<M: Message, F: FnMut(Arc<M>)> ErasedMessageHandler for MessageHandler<M, F> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessageHandlerError> {
        (self.f)(
            msg.as_any_arc()
                .downcast()
                .map_err(|_| MessageHandlerError::WrongMessageType)?,
        );

        Ok(())
    }
}

impl<F: FnMut(Arc<dyn Message>)> ErasedMessageHandler for MessageHandler<dyn Message, F> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessageHandlerError> {
        (self.f)(msg);

        Ok(())
    }
}

impl<H: ErasedMessageHandler + ?Sized> ErasedMessageHandler for &mut H {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessageHandlerError> {
        (**self).call(msg)
    }
}

impl<H: ErasedMessageHandler + ?Sized> ErasedMessageHandler for Box<H> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessageHandlerError> {
        (**self).call(msg)
    }
}

/// A type which can be converted into a [`ErasedMessageHandler`].
pub trait IntoMessageHandler<M: Message + ?Sized>: Sized {
    type Handler: ErasedMessageHandler;

    /// Converts the type into the specific [`ErasedMessageHandler`].
    fn into_message_handler(self) -> Self::Handler;
}

impl<M: Message, F: FnMut(Arc<M>)> IntoMessageHandler<M> for F {
    type Handler = MessageHandler<M, F>;

    fn into_message_handler(self) -> Self::Handler {
        MessageHandler {
            f: self,
            _marker: PhantomData,
        }
    }
}

impl<F: FnMut(Arc<dyn Message>)> IntoMessageHandler<dyn Message> for F {
    type Handler = MessageHandler<dyn Message, F>;

    fn into_message_handler(self) -> Self::Handler {
        MessageHandler {
            f: self,
            _marker: PhantomData,
        }
    }
}

impl<M: Message, F: FnMut(Arc<M>)> IntoMessageHandler<M> for MessageHandler<M, F> {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<F: FnMut(Arc<dyn Message>)> IntoMessageHandler<dyn Message>
    for MessageHandler<dyn Message, F>
{
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<M: Message + ?Sized> IntoMessageHandler<M> for &mut dyn ErasedMessageHandler {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<M: Message + ?Sized> IntoMessageHandler<M> for Box<dyn ErasedMessageHandler> {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

pub enum MessageHandlerError {
    WrongMessageType,
}

/// An iterator which yields messages.
pub trait MessageIterator: Iterator<Item = Arc<dyn Message>> {
    /// Takes a message handler and creates an iterator which
    /// calls that message handler on each received message
    fn handle<'f, M, H>(self, f: impl IntoMessageHandler<M, Handler = H>) -> HandleMessage<'f, Self>
    where
        Self: Sized,
        M: Message + ?Sized,
        H: ErasedMessageHandler + 'f,
    {
        HandleMessage {
            iter: self,
            f: Box::new(f.into_message_handler()),
        }
    }

    /// Runs an iterator.
    fn run(self)
    where
        Self: Sized,
    {
        self.for_each(|_| {});
    }
}

/// An iterator which yields messages from one [`ErasedSubscription`].
///
/// This `struct` is created by [`ErasedSubscription::message_iter`].
pub struct MessageIter<'s> {
    pub(crate) sub: &'s dyn ErasedSubscription,
}

impl Iterator for MessageIter<'_> {
    type Item = Arc<dyn Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sub.recv_message()
    }
}

impl MessageIterator for MessageIter<'_> {}

/// A message iterator which handles messages with `f`.
///
/// This `struct` is created by [`MessageIterator::handle`].
pub struct HandleMessage<'f, I> {
    iter: I,
    f: Box<dyn ErasedMessageHandler + 'f>,
}

impl<I: Iterator<Item = Arc<dyn Message>>> Iterator for HandleMessage<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.iter.next();
        if let Some(msg) = msg.clone() {
            let _ = self.f.call(msg);
        }

        msg
    }
}

impl<I: Iterator<Item = Arc<dyn Message>>> MessageIterator for HandleMessage<'_, I> {}

impl<I: MessageIterator + ?Sized> MessageIterator for Box<I> {}
impl<I: MessageIterator + ?Sized> MessageIterator for &mut I {}
