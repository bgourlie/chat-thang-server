#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;
extern crate serde;
extern crate serde_json;
extern crate clap;
extern crate chrono;
extern crate hyper;

mod message;
mod http_server;
mod ws_server;

use clap::{Arg, App};

const MODE_HTTP: &'static str = "http";
const MODE_WS: &'static str = "ws";

fn main() {
    let matches = App::new("chat-thang-server")
        .version("0.1")
        .author("W. Brian Gourlie <bgourlie@gmail.com>")
        .about("A stupid chat thing")
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .help("The ip address and port to listen on")
            .takes_value(true))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .help("The server mode -- HTTP application server or WebSocket event server")
            .possible_values(&[MODE_HTTP, MODE_WS])
            .takes_value(true))
        .get_matches();

    env_logger::init().unwrap();

    let bind_addr = matches.value_of("bind").unwrap_or("localhost:8080");

    match matches.value_of("mode").unwrap() {
        MODE_HTTP => http_server::listen(&*bind_addr),
        MODE_WS => ws_server::listen(&*bind_addr),
        _ => panic!("Invalid server mode")
    }

    ws_server::listen(&*bind_addr);
}
