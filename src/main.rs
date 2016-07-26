#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;
extern crate serde_json;

use ws::listen;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Message {
    name: String,
    text: String,
}

fn main() {
    env_logger::init().unwrap();

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:2794", |out| {

        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {

            match msg {
                ws::Message::Text(json) => {
                    match serde_json::from_str::<Message>(&json) {
                        Ok(deserialized) => out.send(serde_json::to_string(&deserialized).unwrap()),
                        Err(err) => panic!("Deserialization failed: {:?}", err),
                    }
                }
                ws::Message::Binary(_) => panic!("Not expecting binary data!"),
            }
        }

    }) {
        // Inform the user of failure
        error!("Failed to create WebSocket due to {:?}", error);
    }
}
