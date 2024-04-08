use std::{net::SocketAddr};

use crate::types::Sender;

#[derive(Debug, Clone)]
pub struct Connection {
    id: String,
    address: SocketAddr,
    transmitter: Sender,
}

impl Connection {
    pub fn new(id: String, address: SocketAddr, transmitter: Sender) -> Self {
        Self {
            id,
            address,
            transmitter,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn get_transmitter(&self) -> &Sender {
        &self.transmitter
    }
}
