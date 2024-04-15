use std::io::Cursor;

use crate::utils::read_word;

use super::{
    command::CommandError, DeserializationError, Deserialize, SerializationError, Serialize,
};

pub struct Heartbeat {}

impl Heartbeat {
    pub fn new() -> Self {
        return Heartbeat {};
    }

    pub async fn execute(&self) -> Result<(), CommandError> {
        let time = chrono::Utc::now().timestamp();
        println!("Heartbeat: {}", time);
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
        let mut cursor = Cursor::new(from);
        let word = read_word(&mut cursor)?;

        return match word.as_str() {
            "heartbeat" => Ok(Heartbeat::new()),
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
