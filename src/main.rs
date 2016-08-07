#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;
extern crate serde_json;
extern crate clap;

use ws::{listen, Sender};
use clap::{Arg, App};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Message {
    #[serde(rename="msgType")]
    msg_type: String,
    name: String,
    text: String,
}

fn main() {
    let matches = App::new("chat-thang-server")
        .version("0.1")
        .author("W. Brian Gourlie <bgourlie@gmail.com>")
        .about("A stupid chat thing")
        .arg(Arg::with_name("bind_ip")
            .short("i")
            .long("ip")
            .help("IP address to bind the server to")
            .takes_value(true))
        .arg(Arg::with_name("bind_port")
            .short("p")
            .long("port")
            .help("The port to bind the server to")
            .takes_value(true))
        .get_matches();

    env_logger::init().unwrap();

    let bind_addr = {
        let bind_ip = matches.value_of("bind_ip").unwrap_or("localhost");
        let bind_port = matches.value_of("bind_port").unwrap_or("2794");
        format!("{}:{}", bind_ip, bind_port)
    };

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen(&*bind_addr, |out: Sender| {

        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {

            match msg {
                ws::Message::Text(json) => {
                    match serde_json::from_str::<Message>(&json) {
                        Ok(deserialized) => {
                            out.broadcast(serde_json::to_string(&deserialized).unwrap())
                        }
                        Err(err) => {
                            let err_msg = format!("Deserialization failed: {:?}", err);
                            error!("{}", err_msg);
                            out.send(generate_error(err_msg))
                        }
                    }
                }
                ws::Message::Binary(_) => {
                    let err_msg = "Not expecting binary data!".to_string();
                    error!("{}", err_msg);
                    out.send(generate_error(err_msg))
                }
            }
        }

    }) {
        // Inform the user of failure
        error!("Failed to create WebSocket due to {:?}", error);
    }
}

fn generate_error(message: String) -> String {
    let err = Message {
        msg_type: "error".to_string(),
        name: "error_reporter".to_string(),
        text: message,
    };
    serde_json::to_string(&err).unwrap()
}
