#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;
extern crate serde_json;

use ws::{listen, Sender};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Message {
    #[serde(rename="msgType")]
    msg_type: String,
    name: String,
    text: String,
}

fn main() {
    env_logger::init().unwrap();

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:2794", |out: Sender| {

        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {

            match msg {
                ws::Message::Text(json) => {
                    match serde_json::from_str::<Message>(&json) {
                        Ok(deserialized) => out.broadcast(serde_json::to_string(&deserialized).unwrap()),
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
