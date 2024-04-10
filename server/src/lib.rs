pub mod app;
pub mod websocket;

use std::sync::Arc;

use actix::Addr;
use tokio::sync::Semaphore;
use websocket::server::BotServer;

#[derive(Debug, Clone)]
pub struct AppState {
    server: Addr<BotServer>,
    semaphore: Arc<Semaphore>,
}

impl AppState {
    pub fn new(server: Addr<BotServer>) -> Self {
        return Self {
            server,
            semaphore: Arc::new(Semaphore::new(1000)),
        };
    }

    pub fn server(&self) -> Addr<BotServer> {
        return self.server.clone();
    }

    pub fn semaphore(&self) -> Arc<Semaphore> {
        return self.semaphore.clone();
    }
}
