use lps::*;

use std::{cell::RefCell, sync::Arc};

#[derive(Debug, Clone)]
struct TestMsg0 {
    pub_id: u32,
    msg_id: u32,
}

impl TestMsg0 {
    fn new(pub_id: u32, msg_id: u32) -> Self {
        Self { pub_id, msg_id }
    }
}

impl Message for TestMsg0 {}

#[derive(Debug, Clone)]
struct TestMsg1 {
    pub_id: u32,
    msg_id: u32,
}

impl TestMsg1 {
    fn new(pub_id: u32, msg_id: u32) -> Self {
        Self { pub_id, msg_id }
    }
}

impl Message for TestMsg1 {}

#[derive(Debug, Clone)]
struct TestMsg2 {
    pub_id: u32,
    msg_id: u32,
}

impl TestMsg2 {
    fn new(pub_id: u32, msg_id: u32) -> Self {
        Self { pub_id, msg_id }
    }
}

impl Message for TestMsg2 {}

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
    sub: MultiSubscription,

    data: Vec<(u32, u32, u32)>,
}

impl TestSubsciber0 {
    fn new() -> Self {
        let mut multi_sub = MultiSubscription::unregistered();
        multi_sub.add::<TestMsg0>().add::<TestMsg2>();

        Self {
            sub: multi_sub,

            data: vec![],
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
        let tmp_data = RefCell::new(vec![]);

        self.sub
            .message_iter()
            .handle(|msg: Arc<TestMsg0>| {
                tmp_data.borrow_mut().push((msg.pub_id, 0, msg.msg_id));
            })
            .handle(|msg: Arc<TestMsg2>| {
                tmp_data.borrow_mut().push((msg.pub_id, 2, msg.msg_id));
            })
            .run();

        self.data.append(&mut *tmp_data.borrow_mut());
    }
}

struct TestSubsciber1 {
    sub0: Subscription<TestMsg0>,
    sub1: Subscription<TestMsg1>,

    data: Vec<(u32, u32, u32)>,
}

impl TestSubsciber1 {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            sub0: Subscription::unregistered(),
            sub1: Subscription::unregistered(),

            data: vec![],
        }
    }
}

impl Subscriber for TestSubsciber1 {
    fn subscribe(&mut self, msg_broker: Arc<dyn MessageBroker>) {
        let _ = self.sub0.register(Arc::clone(&msg_broker));
        let _ = self.sub1.register(Arc::clone(&msg_broker));
    }

    fn unsubscribe(&mut self) {
        let _ = self.sub0.unregister();
        let _ = self.sub1.unregister();
    }

    fn activate(&self) {
        let _ = self.sub0.activate();
        let _ = self.sub1.activate();
    }

    fn deactivate(&self) {
        let _ = self.sub0.deactivate();
        let _ = self.sub1.deactivate();
    }

    fn process_messages(&mut self) {
        self.sub0.process_messages(|msg| {
            self.data.push((msg.pub_id, 0, msg.msg_id));
        });
        self.sub1.process_messages(|msg| {
            self.data.push((msg.pub_id, 1, msg.msg_id));
        });
    }
}

#[test]
fn test_single_broker_single_pub_single_sub() {
    let broker: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let pub0 = TestPublisher::new(Arc::clone(&broker));

    let mut sub0 = TestSubsciber0::new();

    {
        pub0.publish(Arc::new(TestMsg0::new(0, 0)));
        pub0.publish(Arc::new(TestMsg1::new(0, 1)));
        pub0.publish(Arc::new(TestMsg2::new(0, 2)));

        sub0.process_messages();
    }

    {
        sub0.subscribe(broker);

        pub0.publish(Arc::new(TestMsg0::new(0, 3)));
        pub0.publish(Arc::new(TestMsg1::new(0, 4)));
        pub0.publish(Arc::new(TestMsg2::new(0, 5)));

        sub0.process_messages();
    }

    {
        sub0.deactivate();

        pub0.publish(Arc::new(TestMsg0::new(0, 6)));
        pub0.publish(Arc::new(TestMsg1::new(0, 7)));
        pub0.publish(Arc::new(TestMsg2::new(0, 8)));

        sub0.process_messages();
    }

    {
        sub0.activate();

        pub0.publish(Arc::new(TestMsg0::new(0, 9)));
        pub0.publish(Arc::new(TestMsg1::new(0, 10)));
        pub0.publish(Arc::new(TestMsg2::new(0, 11)));

        sub0.process_messages();
    }

    {
        sub0.unsubscribe();

        pub0.publish(Arc::new(TestMsg0::new(0, 12)));
        pub0.publish(Arc::new(TestMsg1::new(0, 13)));
        pub0.publish(Arc::new(TestMsg2::new(0, 14)));

        sub0.process_messages();
    }

    assert_eq!((0, 0, 3), sub0.data[0]);
    assert_eq!((0, 2, 5), sub0.data[1]);

    assert_eq!((0, 0, 9), sub0.data[2]);
    assert_eq!((0, 2, 11), sub0.data[3]);

    assert!(sub0.data.get(4).is_none());
}

#[test]
fn test_single_broker_multi_pub_single_sub() {
    let broker: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let pub0 = TestPublisher::new(Arc::clone(&broker));
    let pub1 = TestPublisher::new(Arc::clone(&broker));

    let mut sub0 = TestSubsciber0::new();

    {
        pub0.publish(Arc::new(TestMsg0::new(0, 0)));
        pub0.publish(Arc::new(TestMsg2::new(0, 1)));

        pub1.publish(Arc::new(TestMsg0::new(1, 2)));
        pub1.publish(Arc::new(TestMsg2::new(1, 3)));

        sub0.process_messages();
    }

    {
        sub0.subscribe(broker);

        pub0.publish(Arc::new(TestMsg0::new(0, 4)));
        pub0.publish(Arc::new(TestMsg2::new(0, 5)));

        pub1.publish(Arc::new(TestMsg0::new(1, 6)));
        pub1.publish(Arc::new(TestMsg2::new(1, 7)));

        sub0.process_messages();
    }

    {
        sub0.deactivate();

        pub0.publish(Arc::new(TestMsg0::new(0, 8)));
        pub0.publish(Arc::new(TestMsg2::new(0, 9)));

        pub1.publish(Arc::new(TestMsg0::new(1, 10)));
        pub1.publish(Arc::new(TestMsg2::new(1, 11)));

        sub0.process_messages();
    }

    {
        sub0.activate();

        pub0.publish(Arc::new(TestMsg0::new(0, 12)));
        pub0.publish(Arc::new(TestMsg2::new(0, 13)));

        pub1.publish(Arc::new(TestMsg0::new(1, 14)));
        pub1.publish(Arc::new(TestMsg2::new(1, 15)));

        sub0.process_messages();
    }

    {
        sub0.unsubscribe();

        pub0.publish(Arc::new(TestMsg0::new(0, 16)));
        pub0.publish(Arc::new(TestMsg2::new(0, 17)));

        pub1.publish(Arc::new(TestMsg0::new(1, 18)));
        pub1.publish(Arc::new(TestMsg2::new(1, 19)));

        sub0.process_messages();
    }

    assert_eq!((0, 0, 4), sub0.data[0]);
    assert_eq!((1, 0, 6), sub0.data[1]);
    assert_eq!((0, 2, 5), sub0.data[2]);
    assert_eq!((1, 2, 7), sub0.data[3]);

    assert_eq!((0, 0, 12), sub0.data[4]);
    assert_eq!((1, 0, 14), sub0.data[5]);
    assert_eq!((0, 2, 13), sub0.data[6]);
    assert_eq!((1, 2, 15), sub0.data[7]);

    assert!(sub0.data.get(8).is_none());
}

#[test]
fn test_single_broker_single_pub_multi_sub() {
    let broker: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let pub0 = TestPublisher::new(Arc::clone(&broker));

    let mut sub0 = TestSubsciber0::new();
    let mut sub1 = TestSubsciber1::new();

    {
        pub0.publish(Arc::new(TestMsg0::new(0, 0)));
        pub0.publish(Arc::new(TestMsg1::new(0, 1)));
        pub0.publish(Arc::new(TestMsg2::new(0, 2)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.subscribe(Arc::clone(&broker));
        sub1.subscribe(broker);

        pub0.publish(Arc::new(TestMsg0::new(0, 3)));
        pub0.publish(Arc::new(TestMsg1::new(0, 4)));
        pub0.publish(Arc::new(TestMsg2::new(0, 5)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.deactivate();
        sub1.deactivate();

        pub0.publish(Arc::new(TestMsg0::new(0, 6)));
        pub0.publish(Arc::new(TestMsg1::new(0, 7)));
        pub0.publish(Arc::new(TestMsg2::new(0, 8)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.activate();
        sub1.activate();

        pub0.publish(Arc::new(TestMsg0::new(0, 9)));
        pub0.publish(Arc::new(TestMsg1::new(0, 10)));
        pub0.publish(Arc::new(TestMsg2::new(0, 11)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.unsubscribe();
        sub1.unsubscribe();

        pub0.publish(Arc::new(TestMsg0::new(0, 12)));
        pub0.publish(Arc::new(TestMsg1::new(0, 13)));
        pub0.publish(Arc::new(TestMsg2::new(0, 14)));

        sub0.process_messages();
        sub1.process_messages();
    }

    assert_eq!((0, 0, 3), sub0.data[0]);
    assert_eq!((0, 2, 5), sub0.data[1]);

    assert_eq!((0, 0, 9), sub0.data[2]);
    assert_eq!((0, 2, 11), sub0.data[3]);

    assert!(sub0.data.get(4).is_none());

    assert_eq!((0, 0, 3), sub1.data[0]);
    assert_eq!((0, 1, 4), sub1.data[1]);

    assert_eq!((0, 0, 9), sub1.data[2]);
    assert_eq!((0, 1, 10), sub1.data[3]);

    assert!(sub1.data.get(4).is_none());
}

#[test]
fn test_single_broker_multi_pub_multi_sub() {
    let broker: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let pub0 = TestPublisher::new(Arc::clone(&broker));
    let pub1 = TestPublisher::new(Arc::clone(&broker));

    let mut sub0 = TestSubsciber0::new();
    let mut sub1 = TestSubsciber1::new();

    {
        pub0.publish(Arc::new(TestMsg0::new(0, 0)));
        pub0.publish(Arc::new(TestMsg1::new(0, 1)));
        pub0.publish(Arc::new(TestMsg2::new(0, 2)));

        pub1.publish(Arc::new(TestMsg0::new(1, 0)));
        pub1.publish(Arc::new(TestMsg1::new(1, 1)));
        pub1.publish(Arc::new(TestMsg2::new(1, 2)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.subscribe(Arc::clone(&broker));
        sub1.subscribe(broker);

        pub0.publish(Arc::new(TestMsg0::new(0, 3)));
        pub0.publish(Arc::new(TestMsg1::new(0, 4)));
        pub0.publish(Arc::new(TestMsg2::new(0, 5)));

        pub1.publish(Arc::new(TestMsg0::new(1, 3)));
        pub1.publish(Arc::new(TestMsg1::new(1, 4)));
        pub1.publish(Arc::new(TestMsg2::new(1, 5)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.deactivate();
        sub1.deactivate();

        pub0.publish(Arc::new(TestMsg0::new(0, 6)));
        pub0.publish(Arc::new(TestMsg1::new(0, 7)));
        pub0.publish(Arc::new(TestMsg2::new(0, 8)));

        pub1.publish(Arc::new(TestMsg0::new(1, 6)));
        pub1.publish(Arc::new(TestMsg1::new(1, 7)));
        pub1.publish(Arc::new(TestMsg2::new(1, 8)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.activate();
        sub1.activate();

        pub0.publish(Arc::new(TestMsg0::new(0, 9)));
        pub0.publish(Arc::new(TestMsg1::new(0, 10)));
        pub0.publish(Arc::new(TestMsg2::new(0, 11)));

        pub1.publish(Arc::new(TestMsg0::new(1, 9)));
        pub1.publish(Arc::new(TestMsg1::new(1, 10)));
        pub1.publish(Arc::new(TestMsg2::new(1, 11)));

        sub0.process_messages();
        sub1.process_messages();
    }

    {
        sub0.unsubscribe();
        sub1.unsubscribe();

        pub0.publish(Arc::new(TestMsg0::new(0, 12)));
        pub0.publish(Arc::new(TestMsg1::new(0, 13)));
        pub0.publish(Arc::new(TestMsg2::new(0, 14)));

        pub1.publish(Arc::new(TestMsg0::new(1, 12)));
        pub1.publish(Arc::new(TestMsg1::new(1, 13)));
        pub1.publish(Arc::new(TestMsg2::new(1, 14)));

        sub0.process_messages();
        sub1.process_messages();
    }

    assert_eq!((0, 0, 3), sub0.data[0]);
    assert_eq!((1, 0, 3), sub0.data[1]);
    assert_eq!((0, 2, 5), sub0.data[2]);
    assert_eq!((1, 2, 5), sub0.data[3]);

    assert_eq!((0, 0, 9), sub0.data[4]);
    assert_eq!((1, 0, 9), sub0.data[5]);
    assert_eq!((0, 2, 11), sub0.data[6]);
    assert_eq!((1, 2, 11), sub0.data[7]);

    assert!(sub0.data.get(8).is_none());

    assert_eq!((0, 0, 3), sub1.data[0]);
    assert_eq!((1, 0, 3), sub1.data[1]);
    assert_eq!((0, 1, 4), sub1.data[2]);
    assert_eq!((1, 1, 4), sub1.data[3]);

    assert_eq!((0, 0, 9), sub1.data[4]);
    assert_eq!((1, 0, 9), sub1.data[5]);
    assert_eq!((0, 1, 10), sub1.data[6]);
    assert_eq!((1, 1, 10), sub1.data[7]);

    assert!(sub1.data.get(8).is_none());
}

#[test]
fn test_multi_broker_multi_pub_multi_sub() {
    let broker0: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());
    let broker1: Arc<dyn MessageBroker> = Arc::new(DefaultMessageBroker::new());

    let mut pub0 = TestPublisher::new(Arc::clone(&broker0));
    let mut pub1 = TestPublisher::new(Arc::clone(&broker1));

    let mut sub0 = TestSubsciber0::new();
    let mut sub1 = TestSubsciber1::new();

    for i in 0..2 {
        {
            pub0.publish(Arc::new(TestMsg0::new(0, 0)));
            pub0.publish(Arc::new(TestMsg1::new(0, 1)));
            pub0.publish(Arc::new(TestMsg2::new(0, 2)));

            pub1.publish(Arc::new(TestMsg0::new(1, 0)));
            pub1.publish(Arc::new(TestMsg1::new(1, 1)));
            pub1.publish(Arc::new(TestMsg2::new(1, 2)));

            sub0.process_messages();
            sub1.process_messages();
        }

        {
            sub0.subscribe(Arc::clone(&broker0));
            sub1.subscribe(Arc::clone(&broker1));

            pub0.publish(Arc::new(TestMsg0::new(0, 3)));
            pub0.publish(Arc::new(TestMsg1::new(0, 4)));
            pub0.publish(Arc::new(TestMsg2::new(0, 5)));

            pub1.publish(Arc::new(TestMsg0::new(1, 3)));
            pub1.publish(Arc::new(TestMsg1::new(1, 4)));
            pub1.publish(Arc::new(TestMsg2::new(1, 5)));

            sub0.process_messages();
            sub1.process_messages();
        }

        {
            sub0.deactivate();
            sub1.deactivate();

            pub0.publish(Arc::new(TestMsg0::new(0, 6)));
            pub0.publish(Arc::new(TestMsg1::new(0, 7)));
            pub0.publish(Arc::new(TestMsg2::new(0, 8)));

            pub1.publish(Arc::new(TestMsg0::new(1, 6)));
            pub1.publish(Arc::new(TestMsg1::new(1, 7)));
            pub1.publish(Arc::new(TestMsg2::new(1, 8)));

            sub0.process_messages();
            sub1.process_messages();
        }

        {
            sub0.activate();
            sub1.activate();

            pub0.publish(Arc::new(TestMsg0::new(0, 9)));
            pub0.publish(Arc::new(TestMsg1::new(0, 10)));
            pub0.publish(Arc::new(TestMsg2::new(0, 11)));

            pub1.publish(Arc::new(TestMsg0::new(1, 9)));
            pub1.publish(Arc::new(TestMsg1::new(1, 10)));
            pub1.publish(Arc::new(TestMsg2::new(1, 11)));

            sub0.process_messages();
            sub1.process_messages();
        }

        {
            sub0.unsubscribe();
            sub1.unsubscribe();

            pub0.publish(Arc::new(TestMsg0::new(0, 12)));
            pub0.publish(Arc::new(TestMsg1::new(0, 13)));
            pub0.publish(Arc::new(TestMsg2::new(0, 14)));

            pub1.publish(Arc::new(TestMsg0::new(1, 12)));
            pub1.publish(Arc::new(TestMsg1::new(1, 13)));
            pub1.publish(Arc::new(TestMsg2::new(1, 14)));

            sub0.process_messages();
            sub1.process_messages();
        }

        if i == 0 {
            assert_eq!((0, 0, 3), sub0.data[0]);
            assert_eq!((0, 2, 5), sub0.data[1]);

            assert_eq!((0, 0, 9), sub0.data[2]);
            assert_eq!((0, 2, 11), sub0.data[3]);

            assert!(sub0.data.get(4).is_none());
            sub0.data.clear();

            assert_eq!((1, 0, 3), sub1.data[0]);
            assert_eq!((1, 1, 4), sub1.data[1]);

            assert_eq!((1, 0, 9), sub1.data[2]);
            assert_eq!((1, 1, 10), sub1.data[3]);

            assert!(sub1.data.get(4).is_none());
            sub1.data.clear();

            pub0.set_message_broker(Arc::clone(&broker1));
            pub1.set_message_broker(Arc::clone(&broker0));
        } else {
            assert_eq!((1, 0, 3), sub0.data[0]);
            assert_eq!((1, 2, 5), sub0.data[1]);

            assert_eq!((1, 0, 9), sub0.data[2]);
            assert_eq!((1, 2, 11), sub0.data[3]);

            assert!(sub0.data.get(4).is_none());

            assert_eq!((0, 0, 3), sub1.data[0]);
            assert_eq!((0, 1, 4), sub1.data[1]);

            assert_eq!((0, 0, 9), sub1.data[2]);
            assert_eq!((0, 1, 10), sub1.data[3]);

            assert!(sub1.data.get(4).is_none());
        }
    }
}
