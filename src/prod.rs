use std::time::Duration;

use rabbitmq_stream_client::{
    Environment, NoDedup, Producer,
    error::StreamCreateError,
    types::{Message, ResponseCode},
};

pub struct Prod {
    //environment: Environment,
    producer: Producer<NoDedup>,
}

impl Prod {
    pub async fn new(channel_name: &str) -> Self {
        let environment_result = Environment::builder().build().await;
        if let Err(err) = environment_result {
            panic!("Could not create environment: {}", err);
        }
        let environment = environment_result.unwrap();

        let _ = environment.delete_stream(channel_name).await;
        if let Err(err) = environment.stream_creator().create(channel_name).await {
            if let StreamCreateError::Create { stream, status } = err {
                match status {
                    ResponseCode::StreamAlreadyExists => {}
                    err => {
                        println!("Error creating stream: {:?} {:?}", stream, err);
                    }
                }
            }
        }

        let producer_result = environment.producer().build(channel_name).await;
        if let Err(err) = producer_result {
            panic!("Could not create producer: {}", err);
        }

        Prod {
            //environment: environment,
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
