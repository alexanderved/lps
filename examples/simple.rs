use lps::*;

use std::sync::Arc;

#[derive(Debug, Clone)]
struct TestMsg0(i32);

#[derive(Debug, Clone)]
struct TestMsg1(i32);

impl Message for TestMsg0 {}
impl Message for TestMsg1 {}

struct TestPublisher {
    msg_broker: Arc<dyn MessageBroker>,
}

impl TestPublisher {
    fn new(msg_broker: Arc<dyn MessageBroker>) -> Self {
        Self { msg_broker }
    }
}

impl Publisher for TestPublisher {
    fn message_broker(&self) -> Arc<dyn MessageBroker> {
        Arc::clone(&self.msg_broker)
    }

    fn set_message_broker(&mut self, msg_broker: Arc<dyn MessageBroker>) {
        self.msg_broker = msg_broker;
    }
}

struct TestSubsciber0 {
    sub: Subscription<TestMsg0>,
}

impl TestSubsciber0 {
    fn new() -> Self {
        Self {
            sub: Subscription::unregistered(),
        }
    }
}

impl Subscriber for TestSubsciber0 {
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
            .handle(|msg: Arc<TestMsg0>| {
                println!("test_sub0.sub: {:?}", msg);
            })
            .run();
    }
}

struct TestSubsciber1 {
    sub: MultiSubscription,
}

impl TestSubsciber1 {
    fn new() -> Self {
        let mut multi_sub = MultiSubscription::unregistered();
        multi_sub.add::<TestMsg0>().add::<TestMsg1>();

        Self { sub: multi_sub }
    }
}

impl Subscriber for TestSubsciber1 {
    fn subscribe(&mut self, msg_broker: Arc<dyn MessageBroker>) {
        let _ = self.sub.register(Arc::clone(&msg_broker));
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
            .handle(|msg: Arc<TestMsg0>| {
                println!("test_sub1.sub0: {:?}", msg);
            })
            .handle(|msg: Arc<TestMsg1>| {
                println!("test_sub1.sub1: {:?}", msg);
            })
            .run();
    }
}

fn main() {
    let tb: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let mut s0 = TestSubsciber0::new();
    s0.subscribe(Arc::clone(&tb));

    let mut s1 = TestSubsciber1::new();
    s1.subscribe(Arc::clone(&tb));

    let p0 = TestPublisher::new(Arc::clone(&tb));
    let p1 = TestPublisher::new(Arc::clone(&tb));

    p0.publish(Arc::new(TestMsg0(2)));
    s1.deactivate();
    p1.publish(Arc::new(TestMsg0(5)));
    s1.activate();
    p1.publish(Arc::new(TestMsg1(3)));
    p1.publish(Arc::new(TestMsg1(6)));

    s0.process_messages();
    s1.process_messages();
}
