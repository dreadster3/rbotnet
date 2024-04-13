use std::io::Cursor;

use crate::utils::read_word;

use super::{DeserializationError, Deserialize, SerializationError, Serialize};

pub struct Heartbeat {}

impl Heartbeat {
    pub fn new() -> Self {
        return Heartbeat {};
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Heartbeat");
        return Ok(());
    }
}

impl Serialize for Heartbeat {
    fn serialize(&self) -> Result<String, SerializationError> {
        return Ok("heartbeat".to_string());
    }
}

impl Deserialize for Heartbeat {
    fn deserialize(from: String) -> Result<Self, DeserializationError> {
        let cursor = Cursor::new(from);
        let word = read_word(cursor)?;

        return match word.as_str() {
            "heartbeat" => Ok(Heartbeat::new()),
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
