use super::{
    heartbeat::Heartbeat, request::Request, DeserializationError, Deserialize, SerializationError,
    Serialize,
};
use crate::utils::read_word;
use std::io::Cursor;

#[derive(Debug)]
pub enum CommandError {}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "Command error");
    }
}

impl std::error::Error for CommandError {}

type Result<T> = std::result::Result<T, CommandError>;

pub enum Command {
    Heartbeat(Heartbeat),
    Request(Request),
}

impl Command {
    pub async fn execute(&self) -> Result<()> {
        return match self {
            Command::Heartbeat(c) => c.execute().await,
            Command::Request(c) => c.execute().await,
        };
    }
}

impl Serialize for Command {
    fn serialize(&self) -> std::result::Result<String, SerializationError> {
        return match self {
            Command::Heartbeat(c) => c.serialize(),
            Command::Request(c) => c.serialize(),
        };
    }
}

impl Deserialize for Command {
    fn deserialize(from: String) -> std::result::Result<Self, DeserializationError> {
        let mut cursor = Cursor::new(from.clone());
        let word = read_word(&mut cursor)?;

        println!("Deserializing command: {}", word.clone());

        return match word.as_str() {
            "heartbeat" => match Heartbeat::deserialize(from.clone()) {
                Ok(h) => Ok(Command::Heartbeat(h)),
                Err(e) => Err(e),
            },
            "httprequest" => match Request::deserialize(from.clone()) {
                Ok(r) => Ok(Command::Request(r)),
                Err(e) => Err(e),
            },
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
