use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::connection::Connection;

#[derive(Debug, Clone)]
pub enum ContextDataType {
    String(String),
    Integer(i32),
    Float(f32),
    Connections(Arc<Mutex<HashMap<String, Connection>>>),
}

pub type Context = HashMap<String, ContextDataType>;

pub mod known_keys {
    pub const CONNECTIONS_KEY: &str = "connections";
}

impl From<&str> for ContextDataType {
    fn from(s: &str) -> Self {
        ContextDataType::String(s.to_string())
    }
}

impl From<String> for ContextDataType {
    fn from(s: String) -> Self {
        ContextDataType::String(s)
    }
}

impl From<i32> for ContextDataType {
    fn from(i: i32) -> Self {
        ContextDataType::Integer(i)
    }
}

impl From<f32> for ContextDataType {
    fn from(f: f32) -> Self {
        ContextDataType::Float(f)
    }
}

impl From<Arc<Mutex<HashMap<String, Connection>>>> for ContextDataType {
    fn from(connections: Arc<Mutex<HashMap<String, Connection>>>) -> Self {
        ContextDataType::Connections(connections)
    }
}
