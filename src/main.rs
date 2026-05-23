mod community;
mod map;
mod moon_walker;
mod moon_walker_out_comm;
mod server;

use std::process::exit;

use crate::moon_walker::MoonWalker;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Program type unspecified");
        exit(1);
    }

    match args[1].as_str() {
        "moon-walker" => {
            MoonWalker::run_moon_walker();
        }
        "server" => {
            println!("server started");
        }
        "community" => {
            println!("community started");
        }
        _ => {
            println!("No such type");
            exit(1);
        }
    }
}
