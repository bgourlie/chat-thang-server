#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ws;
extern crate serde;
extern crate serde_json;
extern crate clap;
extern crate time;

use ws::{listen, Sender};
use clap::{Arg, App};
use time::Tm as Time;


#[derive(Debug, PartialEq)]
struct Message {
    msg_type: String,
    name: String,
    text: String,
    time: Time,
}

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let time_string = format!("{}", self.time.rfc3339());
        let mut state = try!(serializer.serialize_struct("Message", 4));
        try!(serializer.serialize_struct_elt(&mut state, "msgType", &*self.msg_type));
        try!(serializer.serialize_struct_elt(&mut state, "name", &*self.name));
        try!(serializer.serialize_struct_elt(&mut state, "text", &*self.text));
        try!(serializer.serialize_struct_elt(&mut state, "time", &*time_string));
        serializer.serialize_struct_end(state)
    }
}

impl serde::Deserialize for Message {
    fn deserialize<D>(deserializer: &mut D) -> Result<Message, D::Error>
        where D: serde::de::Deserializer
    {
        static FIELDS: &'static [&'static str] = &["msgType", "name", "text", "time"];
        deserializer.deserialize_struct("Message", FIELDS, MessageVisitor)
    }
}

enum MessageField {
    MessageType,
    Name,
    Text,
    Time,
}

impl serde::Deserialize for MessageField {
    fn deserialize<D>(deserializer: &mut D) -> Result<MessageField, D::Error>
        where D: serde::de::Deserializer
    {
        struct MessageFieldVisitor;

        impl serde::de::Visitor for MessageFieldVisitor {
            type Value = MessageField;

            fn visit_str<E>(&mut self, value: &str) -> Result<MessageField, E>
                where E: serde::de::Error
            {
                match value {
                    "msgType" => Ok(MessageField::MessageType),
                    "name" => Ok(MessageField::Name),
                    "text" => Ok(MessageField::Text),
                    "time" => Ok(MessageField::Time),
                    _ => Err(serde::de::Error::custom("Unexpected field name encountered")),
                }
            }
        }

        deserializer.deserialize(MessageFieldVisitor)
    }
}

struct MessageVisitor;

impl serde::de::Visitor for MessageVisitor {
    type Value = Message;

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<Message, V::Error>
        where V: serde::de::MapVisitor
    {
        let mut msg_type = None;
        let mut name = None;
        let mut text = None;
        let mut time: Option<String> = None;

        loop {
            match try!(visitor.visit_key()) {
                Some(MessageField::MessageType) => {
                    msg_type = Some(try!(visitor.visit_value()));
                }
                Some(MessageField::Name) => {
                    name = Some(try!(visitor.visit_value()));
                }
                Some(MessageField::Text) => {
                    text = Some(try!(visitor.visit_value()));
                }
                Some(MessageField::Time) => {
                    time = Some(try!(visitor.visit_value()));
                }
                None => {
                    break;
                }
            }
        }

        let msg_type = match msg_type {
            Some(msg_type) => msg_type,
            None => try!(visitor.missing_field("msgType")),
        };

        let name = match name {
            Some(name) => name,
            None => try!(visitor.missing_field("name")),
        };

        let text = match text {
            Some(text) => text,
            None => try!(visitor.missing_field("text")),
        };

        let time = match time {
            Some(time) => time,
            None => try!(visitor.missing_field("time")),
        };

        let time_struct = match time::strptime(&*time, "%+") {
            Ok(time_struct) => time_struct,
            Err(_) => try!(Err(serde::de::Error::custom("Malformed time format"))),
        };

        try!(visitor.end());

        Ok(Message {
            msg_type: msg_type,
            name: name,
            text: text,
            time: time_struct,
        })
    }
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
        let bind_port = matches.value_of("bind_port").unwrap_or("8080");
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
                            warn!("{}", err_msg);
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
        time: time::now_utc(),
    };
    serde_json::to_string(&err).unwrap()
}
