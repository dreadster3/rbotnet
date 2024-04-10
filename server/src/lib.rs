pub mod app;
pub mod websocket;

use std::sync::Arc;

use actix::{dev::channel::AddressSender, Addr};
use websocket::server::BotServer;

#[derive(Debug, Clone)]
pub struct AppState {
    server: Addr<BotServer>,
}

impl AppState {
    pub fn new(server: Addr<BotServer>) -> Self {
        return Self { server };
    }

    pub fn server(&self) -> Addr<BotServer> {
        return self.server.clone();
    }
}
