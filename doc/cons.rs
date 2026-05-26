...
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
...
