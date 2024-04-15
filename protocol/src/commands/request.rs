use std::io::{BufRead, Cursor};

use crate::utils::read_word;

use super::{command::CommandError, DeserializationError, Deserialize, Serialize};

pub struct Request {
    method: String,
    path: String,
    body: Option<String>,
}

impl Request {
    pub fn new(method: &str, path: &str, body: Option<String>) -> Self {
        return Request {
            method: method.to_string(),
            path: path.to_string(),
            body,
        };
    }

    pub async fn execute(&self) -> Result<(), CommandError> {
        println!(
            "Request: {} {} {}",
            self.method,
            self.path,
            self.body.clone().unwrap_or("".to_string())
        );
        return Ok(());
    }
}

impl Serialize for Request {
    fn serialize(&self) -> Result<String, super::SerializationError> {
        let mut parameters = vec![self.method.clone(), self.path.clone()];
        if let Some(body) = &self.body {
            parameters.push(body.clone());
        }

        return Ok(format!("httprequest {}", parameters.join(" ")));
    }
}

impl Deserialize for Request {
    fn deserialize(from: String) -> Result<Self, super::DeserializationError> {
        let mut cursor = Cursor::new(from);
        let word = read_word(&mut cursor)?;

        return match word.as_str() {
            "httprequest" => {
                let method = read_word(&mut cursor)?;
                let path = read_word(&mut cursor)?;
                let mut body = Some(String::new());
                if let Err(_) = cursor.read_line(&mut body.as_mut().unwrap()) {
                    body = None;
                };

                Ok(Request::new(&method, &path, body))
            }
            _ => Err(DeserializationError::InvalidCommand),
        };
    }
}
