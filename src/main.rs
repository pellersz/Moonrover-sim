mod client;
mod cons;
mod map;
mod moon_walker;
mod prod;
mod server;

use std::process::exit;

use crate::{
    client::{ClientControl, ClientUI},
    moon_walker::MoonWalker,
    server::Server,
};

// TODO: do something with the casting
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
            ClientUI::run_client_ui().await;
        }
        "client-cli" => {
            ClientControl::run_client_control().await;
        }
        _ => {
            println!("No such type");
            exit(1);
        }
    }
}
