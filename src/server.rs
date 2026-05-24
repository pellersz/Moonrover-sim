use crate::{cons::Cons, prod::Prod};

pub struct Server {
    moon_walker_consumer: Cons,
    client_producer: Prod,
}

impl Server {
    pub async fn run_server() {
        let mut server = Server {
            moon_walker_consumer: Cons::new("moon-walker-out").await,
            client_producer: Prod::new("server-client-out").await,
        };

        let first_message = String::from_utf8(server.moon_walker_consumer.read().await).unwrap();
        let mut splits = first_message.split(|x| x == ',');
        for _ in 0..4 {
            splits.next();
        }
        server
            .client_producer
            .send(&format!("i:{}", splits.next().unwrap()))
            .await;

        loop {
            match str::from_utf8(server.moon_walker_consumer.read().await.as_slice()) {
                Ok(str) => {
                    if str.len() != 0 {
                        println!("got a message from the moon walker: {}", str);
                        if str.as_bytes()[0] != '(' as u8 {
                            server
                                .client_producer
                                .send(&format!("s:{}", &str[0..str.rfind(|x| x == ',').unwrap()]))
                                .await;
                        } else {
                            server.client_producer.send(&format!("p:{}", str)).await;
                        }
                    }
                }
                Err(err) => {
                    println!("error {}", err)
                }
            }
        }
    }
}
