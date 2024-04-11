pub mod command;

#[derive(Debug)]
pub enum SerializationError {
    IOError(std::io::Error),
    InvalidCommand,
}

#[derive(Debug)]
pub enum DeserializationError {
    IOError(std::io::Error),
    InvalidCommand,
}

pub trait Serialize {
    fn serialize(&self) -> Result<String, SerializationError>;
}

pub trait Deserialize {
    fn deserialize(from: String) -> Result<Self, DeserializationError>
    where
        Self: Sized;
}

impl From<std::io::Error> for DeserializationError {
    fn from(error: std::io::Error) -> Self {
        return DeserializationError::IOError(error);
    }
}

impl From<std::io::Error> for SerializationError {
    fn from(error: std::io::Error) -> Self {
        return SerializationError::IOError(error);
    }
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {
            SerializationError::IOError(error) => write!(f, "IO error: {}", error),
            SerializationError::InvalidCommand => write!(f, "Invalid command"),
        };
    }
}

impl std::error::Error for SerializationError {}

impl std::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {
            DeserializationError::IOError(error) => write!(f, "IO error: {}", error),
            DeserializationError::InvalidCommand => write!(f, "Invalid command"),
        };
    }
}

impl std::error::Error for DeserializationError {}
