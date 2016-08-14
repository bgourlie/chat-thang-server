use chrono::{UTC, DateTime};
use serde::{Serialize, Serializer};
use serde;
use serde_json;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub msg_type: String,
    pub name: String,
    pub text: String,
    pub time: DateTime<UTC>,
}

impl Message {
    pub fn with_error(message: String) -> Self {
        Message {
            msg_type: "error".to_string(),
            name: "error_reporter".to_string(),
            text: message,
            time: UTC::now(),
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        // Not sure what could cause this to fail, but if it does, return an
        // empty object instead of crashing.  This will cause a deserialization
        // error on the client unless we handle this.
        serde_json::to_string(&self).unwrap_or("{}".to_string())
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let time_string = self.time.to_rfc3339();
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

        let time_struct = match DateTime::parse_from_rfc3339(&*time) {
            Ok(time_struct) => time_struct.with_timezone(&UTC),
            Err(_) => {
                try!(Err(serde::de::Error::custom(format!("Malformed time field: {}", time))))
            }
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
