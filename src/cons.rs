use futures::StreamExt;
use rabbitmq_stream_client::{Consumer, Environment, types::OffsetSpecification};

use crate::prod::Prod;

pub struct Cons {
    environment: Environment,
    consumer: Consumer,
    channel_name: String,
}

impl Cons {
    pub async fn new(channel_name: &str) -> Self {
        let environment_result = Environment::builder().build().await;
        if let Err(err) = environment_result {
            panic!("Could not create environment: {}", err);
        }
        let environment = environment_result.unwrap();

        let consumer_result = environment
            .consumer()
            .offset(OffsetSpecification::First)
            .build(channel_name)
            .await;
        if let Err(err) = consumer_result {
            panic!("Could not create stream: {}", err);
        }

        Cons {
            environment: environment,
            consumer: consumer_result.unwrap(),
            channel_name: channel_name.to_owned(),
        }
    }

    pub async fn read(&mut self) -> Vec<u8> {
        if let Some(delivery) = self.consumer.next().await {
            if let Err(err) = delivery {
                println!("Error happened with a delivery: {}", err);
                return Vec::new();
            }

            return delivery.unwrap().message().data().unwrap().to_vec();
        }

        return Vec::new();
    }

    pub async fn move_offset_curr(&mut self) {
        let consumer_result = self
            .environment
            .consumer()
            .offset(OffsetSpecification::Next)
            .build(&self.channel_name)
            .await;
        if let Err(err) = consumer_result {
            panic!("Could not create stream: {}", err);
        }

        self.consumer = consumer_result.unwrap();
    }
}
