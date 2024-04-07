use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use super::Sender;

pub struct Connection {
    id: String,
    address: SocketAddr,
    sender: Sender,
}

impl Connection {
    pub fn new(id: String, address: SocketAddr, sender: Sender) -> Self {
        Self {
            id,
            address,
            sender,
        }
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn get_sender(&self) -> &Sender {
        &self.sender
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}

pub type State = Arc<Mutex<HashMap<String, Connection>>>;

pub fn new() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn add_connection(state: State, connection: Connection) {
    let mut state = state.lock().await;
    state.insert(connection.get_id().to_string(), connection);
}
