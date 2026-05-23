use std::time::Duration;

use rabbitmq_stream_client::{Environment, NoDedup, Producer, types::Message};

pub struct MoonWalkerOutCommunicator {
    environment: Environment,
    producer: Producer<NoDedup>,
}

impl MoonWalkerOutCommunicator {
    const CHANNEL_NAME: &str = "moon-walker-out";

    pub async fn new() -> Self {
        let environment_result = Environment::builder().build().await;
        if let Err(err) = environment_result {
            panic!("Could not create environment: {}", err);
        }
        let environment = environment_result.unwrap();

        if let Err(err) = environment
            .stream_creator()
            .max_segment_size(rabbitmq_stream_client::types::ByteCapacity::GB(5))
            .max_age(Duration::from_mins(10))
            .create(MoonWalkerOutCommunicator::CHANNEL_NAME)
            .await
        {
            panic!("Could not create stream: {}", err);
        }

        let producer_result = environment
            .producer()
            .build(MoonWalkerOutCommunicator::CHANNEL_NAME)
            .await;
        if let Err(err) = producer_result {
            panic!("Could not create stream: {}", err);
        }

        MoonWalkerOutCommunicator {
            environment: environment,
            producer: producer_result.unwrap(),
        }
    }

    pub async fn send(&self, msg: &str) {
        let _ = self
            .producer
            .send_with_confirm(Message::builder().body(msg).build())
            .await;
        return;
    }
}
