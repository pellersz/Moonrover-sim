mod client;
mod community;
mod cons;
mod map;
mod moon_walker;
mod prod;
mod server;

use std::process::exit;

use crate::{client::Client, moon_walker::MoonWalker, server::Server};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Program type unspecified");
        exit(1);
    }

    match args[1].as_str() {
        "moon-walker" => {
            MoonWalker::run_moon_walker().await;
        }
        "server" => {
            Server::run_server().await;
        }
        "client-ui" => {
            Client::run_client().await;
        }
        "client-cli" => {
            println!("community started");
        }
        _ => {
            println!("No such type");
            exit(1);
        }
    }
}
