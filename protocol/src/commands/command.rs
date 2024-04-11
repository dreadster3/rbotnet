use super::{DeserializationError, Deserialize, SerializationError, Serialize};
use crate::utils::read_word;
use std::io::Cursor;

pub enum Command {
    Heartbeat,
}

impl Serialize for Command {
    fn serialize(&self) -> Result<String, SerializationError> {
        return match self {
            Command::Heartbeat => Ok("heartbeat".to_string()),
        };
    }
}

impl Deserialize for Command {
    fn deserialize(from: String) -> Result<Self, DeserializationError> {
        let cursor = Cursor::new(from);
        let word = read_word(cursor)?;

        return match word.as_str() {
            "heartbeat" => Ok(Command::Heartbeat),
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
