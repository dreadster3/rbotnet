use super::{
    heartbeat::Heartbeat, DeserializationError, Deserialize, SerializationError, Serialize,
};
use crate::utils::read_word;
use std::io::Cursor;

pub enum Command {
    Heartbeat(Heartbeat),
}

impl Command {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        return match self {
            Command::Heartbeat(c) => c.execute().await,
        };
    }
}

impl Serialize for Command {
    fn serialize(&self) -> Result<String, SerializationError> {
        return match self {
            Command::Heartbeat(c) => c.serialize(),
        };
    }
}

impl Deserialize for Command {
    fn deserialize(from: String) -> Result<Self, DeserializationError> {
        let cursor = Cursor::new(from.clone());
        let word = read_word(cursor)?;

        return match word.as_str() {
            "heartbeat" => match Heartbeat::deserialize(from.clone()) {
                Ok(h) => Ok(Command::Heartbeat(h)),
                Err(e) => Err(e),
            },
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
