use ws;
use ws::{Sender};
use message::Message;
use serde_json;


pub fn listen(bind_addr: &str) {
    // Listen on an address and call the closure for each connection
    if let Err(error) = ws::listen(bind_addr, |out: Sender| {

        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {

            match msg {
                ws::Message::Text(json) => {
                    match serde_json::from_str::<Message>(&json) {
                        Ok(deserialized) => {
                            out.broadcast(deserialized.to_string())
                        }
                        Err(err) => {
                            let err_msg = format!("Deserialization failed: {:?}", err);
                            warn!("{}", err_msg);
                            let msg = Message::with_error(err_msg);
                            out.send(msg.to_string())
                        }
                    }
                }
                ws::Message::Binary(_) => {
                    let err_msg = "Not expecting binary data!".to_string();
                    error!("{}", err_msg);
                    let msg = Message::with_error(err_msg);
                    out.send(msg.to_string())
                }
            }
        }

    }) {
        // Inform the user of failure
        error!("Failed to create WebSocket due to {:?}", error);
    }
}
