#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;

use ws::listen;

fn main() {
    env_logger::init().unwrap();

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:2794", |out| {

        // The handler needs to take ownership of out, so we use move
        move |msg| {

            // Handle messages received on this connection
            info!("Server received: {}", msg);

            // Use the out channel to send messages back
            out.send(msg)
        }

    }) {
        // Inform the user of failure
        error!("Failed to create WebSocket due to {:?}", error);
    }
}
