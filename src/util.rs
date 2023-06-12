use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

/// Downcasts [`Arc<dyn Message>`] to the one of the given types and runs the code
/// which corresponds to it.
///
/// [`Arc<dyn Message>`]: crate::message::Message
#[macro_export]
macro_rules! match_message {
    ($msg:ident { $( $msg_tt:tt )* }) => {
        {
            match_message!(@arm $msg as $( $msg_tt )*);
        }
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:block $(,)?) => {
        use $crate::AsAny;
        use ::std::any::Any;
        if let Ok($msg) = ::std::sync::Arc::clone(&$msg).as_any_arc().downcast::<$msg_ty>() {
            $msg_handler;
        }
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:expr $(,)?) => {
        match_message!(@arm $msg as $msg_ty => { $msg_handler; });
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:block, $( $msg_tt:tt )*) => {
        match_message!(@arm $msg as $msg_ty => $msg_handler);
        match_message!(@arm $msg as $( $msg_tt )*);
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:expr, $( $msg_tt:tt )*) => {
        match_message!(@arm $msg as $msg_ty => { $msg_handler; }, $( $msg_tt )*);
    };
}

#[doc(hidden)]
pub trait AsAny {
    #[doc(hidden)]
    fn as_any_ref(&self) -> &dyn Any;
    #[doc(hidden)]
    fn as_any_mut(&mut self) -> &mut dyn Any;
    #[doc(hidden)]
    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
    #[doc(hidden)]
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
    #[doc(hidden)]
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync;
}

impl<T: 'static> AsAny for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync,
    {
        self
    }
}
