use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use actix::{
    dev::ContextFutureSpawner, Actor, ActorFutureExt, AsyncContext, Handler, Recipient,
    ResponseActFuture, WrapFuture,
};
use log::info;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use super::messages::{Connected, Disconnected, ListSessions, Message};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BotConnection {
    id: String,
    address: SocketAddr,

    #[serde(skip)]
    recipient: Recipient<Message>,
}

#[derive(Debug)]
pub struct BotServer {
    sessions: Arc<Mutex<HashMap<String, BotConnection>>>,
}

impl BotServer {
    pub fn new() -> Self {
        return Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    pub fn sessions(&self) -> Arc<Mutex<HashMap<String, BotConnection>>> {
        return self.sessions.clone();
    }
}

impl Actor for BotServer {
    type Context = actix::Context<Self>;
}

impl Handler<Connected> for BotServer {
    type Result = Result<String, Box<dyn std::error::Error + Sync + Send>>;

    fn handle(&mut self, msg: Connected, ctx: &mut Self::Context) -> Self::Result {
        info!("Connected: {:?}", msg.address);
        let sessions = self.sessions.clone();
        let message_id = msg.id.clone();

        async move {
            let connection = BotConnection {
                id: message_id.clone(),
                address: msg.address,
                recipient: msg.recipient,
            };

            let mut sessions = sessions.lock().await;
            sessions.insert(message_id, connection);
        }
        .into_actor(self)
        .wait(ctx);

        return Ok(msg.id);
    }
}

impl Handler<Disconnected> for BotServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnected, ctx: &mut Self::Context) {
        info!("Disconnected: {:?}", msg.id);
        let sessions = self.sessions();

        async move {
            let mut sessions = sessions.lock().await;
            sessions.remove(&msg.id);
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Handler<ListSessions> for BotServer {
    type Result = ResponseActFuture<Self, Vec<BotConnection>>;

    fn handle(&mut self, _: ListSessions, _: &mut Self::Context) -> Self::Result {
        let sessions = self.sessions();

        async move {
            let sessions = sessions.lock().await;
            sessions.values().cloned().collect()
        }
        .into_actor(self)
        .boxed_local()
    }
}
