use std::time::{Duration, Instant};

use crate::{cons::Cons, prod::Prod};

pub struct Server {
    moon_walker_consumer: Cons,
    client_consumer: Cons,
    moon_walker_producer: Prod,
    client_producer: Prod,
    moon_dim: i16,
}

impl Server {
    pub async fn run_server() {
        let mut server = Server {
            moon_walker_producer: Prod::new("moon-walker-in").await,
            client_producer: Prod::new("server-client-out").await,
            moon_walker_consumer: Cons::new("moon-walker-out").await,
            client_consumer: Cons::new("client-votes").await,
            moon_dim: 0,
        };

        let first_message =
            String::from_utf8(server.moon_walker_consumer.read_non_blocking().await).unwrap();
        println!("received from the moonwalker: {}", &first_message);
        let mut splits = first_message.split(|x| x == ',');
        for _ in 0..4 {
            splits.next();
        }

        server.moon_dim = splits.next().unwrap().parse::<i16>().unwrap();
        let message = format!("i:{}", server.moon_dim);
        server.client_producer.send(&message).await;
        println!("sent to the client: {}", &message);

        server.moon_walker_consumer.move_offset_curr().await;

        let mut start = Instant::now();

        loop {
            match str::from_utf8(
                server
                    .moon_walker_consumer
                    .read_non_blocking()
                    .await
                    .as_slice(),
            ) {
                Ok(message) => {
                    if message.len() != 0 {
                        println!("received from the moonwalker: {}", message);
                        if message.as_bytes()[0] != '(' as u8 {
                            let message =
                                format!("s:{}", &message[0..message.rfind(|x| x == ',').unwrap()]);
                            server.client_producer.send(&message).await;
                            println!("sent to the client: {}", &message);
                        } else {
                            let message = format!("p:{}", message);
                            server.client_producer.send(&message).await;
                            println!("sent to the client: {}", &message)
                        }
                    }
                }
                Err(err) => {
                    println!("error {}", err)
                }
            }

            if Instant::now() - start > Duration::from_secs(10) {
                start = Instant::now();

                server
                    .client_consumer
                    .move_offset_duration(Duration::from_secs(10))
                    .await;

                let mut votes: Vec<(i16, i16)> = Vec::new();
                loop {
                    match str::from_utf8(
                        server.client_consumer.read_non_blocking().await.as_slice(),
                    ) {
                        Ok(str) => {
                            if str.len() != 0 {
                                println!("received from the client: {}", str);

                                let mut nums = str.split(|x| x == ' ');
                                if let (Some(first), Some(second)) = (nums.next(), nums.next()) {
                                    if let (Ok(x), Ok(y)) =
                                        (first.parse::<i16>(), second.parse::<i16>())
                                    {
                                        if x < server.moon_dim && y < server.moon_dim {
                                            votes.push((x, y));
                                        }
                                    } else {
                                        println!("Wrong numbers");
                                    }
                                } else {
                                    println!("Wrong numbers");
                                }
                            } else {
                                break;
                            }
                        }
                        Err(err) => {
                            println!("error {}", err)
                        }
                    }
                }

                if votes.len() != 0 {
                    let (x_sum, y_sum) = votes
                        .iter()
                        .fold((0_usize, 0_usize), |(x_acc, y_acc), &(x, y)| {
                            (x_acc + x as usize, y_acc + y as usize)
                        });

                    let message = format!("{} {}", x_sum / votes.len(), y_sum / votes.len());
                    server.moon_walker_producer.send(&message).await;
                    println!("sent the moonwalker: {}", &message);
                }
            }
        }
    }
}
