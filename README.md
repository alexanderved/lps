# lps
A local publish-subscribe pattern implementation

## Example
```rust
use lps::*;

use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct Msg(i32);

impl Message for Msg {}

struct Pub {
    msg_broker: Arc<dyn MessageBroker>,
}

impl Pub {
    fn new(msg_broker: Arc<dyn MessageBroker>) -> Self {
        Self { msg_broker }
    }
}

impl Publisher for Pub {
    fn message_broker(&self) -> Arc<dyn MessageBroker> {
        Arc::clone(&self.msg_broker)
    }

    fn set_message_broker(&mut self, msg_broker: Arc<dyn MessageBroker>) {
        self.msg_broker = msg_broker;
    }
}

struct Sub {
    sub: Subscription<Msg>,
}

impl Sub {
    fn new() -> Self {
        Self {
            sub: Subscription::unregistered(),
        }
    }
}

impl Subscriber for Sub {
    fn subscribe(&mut self, msg_broker: Arc<dyn MessageBroker>) {
        let _ = self.sub.register(msg_broker);
    }

    fn unsubscribe(&mut self) {
        let _ = self.sub.unregister();
    }

    fn activate(&self) {
        let _ = self.sub.activate();
    }

    fn deactivate(&self) {
        let _ = self.sub.deactivate();
    }

    fn process_messages(&mut self) {
        self.sub
            .message_iter()
            .handle(|msg: Arc<Msg>| {
                println!("The subscriber received a message: {:?}", msg);
            })
            .run();
    }
}

let msg_broker: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

let mut s = Sub::new();
s.subscribe(Arc::clone(&msg_broker));

let p = Pub::new(Arc::clone(&msg_broker));

p.publish(Arc::new(Msg(1)));
p.publish(Arc::new(Msg(2)));
p.publish(Arc::new(Msg(3)));

s.process_messages();

```
