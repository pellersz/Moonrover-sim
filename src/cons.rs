use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{StreamExt, TryStreamExt};
use rabbitmq_stream_client::{Consumer, Environment, types::OffsetSpecification};

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

        let mut consumer_result;
        loop {
            consumer_result = environment
                .consumer()
                .offset(OffsetSpecification::First)
                .build(channel_name)
                .await;

            if let Ok(_) = consumer_result {
                break;
            }
        }

        Cons {
            environment: environment,
            consumer: consumer_result.unwrap(),
            channel_name: channel_name.to_owned(),
        }
    }

    // TODO: make this optional
    pub async fn read_non_blocking(&mut self) -> Vec<u8> {
        if let Ok(Ok(Some(delivery))) =
            tokio::time::timeout(Duration::from_millis(10), self.consumer.try_next()).await
        {
            return delivery.message().data().unwrap().to_vec();
        }

        return Vec::new();
    }

    // TODO: same with this
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

    pub async fn move_offset_duration(&mut self, duration: Duration) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            - duration.as_millis() as i64;

        let consumer_result = self
            .environment
            .consumer()
            .offset(OffsetSpecification::Timestamp(timestamp))
            .build(&self.channel_name)
            .await;

        if let Err(err) = consumer_result {
            panic!("Could not create stream: {}", err);
        }

        self.consumer = consumer_result.unwrap();
    }
}
