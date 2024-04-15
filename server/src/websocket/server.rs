use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use actix::{
    dev::ContextFutureSpawner, Actor, ActorFutureExt, AsyncContext, Handler, Recipient,
    ResponseActFuture, WrapFuture,
};
use futures::FutureExt;
use log::info;
use serde::Serialize as JsonSerialize;
use tokio::sync::Mutex;

use super::messages::{
    BroadcastCommand, Connected, Disconnect, Disconnected, ListSessions, Message, SendCommand,
};
use protocol::commands::{Deserialize, Serialize};

#[derive(Debug, Clone, JsonSerialize)]
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
        let session_id = msg.0.clone();
        info!("Disconnected: {:?}", session_id);
        let sessions = self.sessions();

        async move {
            let mut sessions = sessions.lock().await;
            sessions.remove(&session_id.clone());
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

impl<T> Handler<BroadcastCommand<T>> for BotServer
where
    T: Serialize + Deserialize,
{
    type Result = super::Result<()>;

    fn handle(&mut self, msg: BroadcastCommand<T>, ctx: &mut Self::Context) -> Self::Result {
        let sessions = self.sessions();
        let serialized_msg = msg.0.serialize()?;

        async move {
            let sessions = sessions.lock().await;
            for session in sessions.values() {
                session.recipient.do_send(Message(serialized_msg.clone()));
            }
        }
        .into_actor(self)
        .wait(ctx);

        Ok(())
    }
}

impl<T> Handler<SendCommand<T>> for BotServer
where
    T: Serialize + Deserialize,
{
    type Result = ResponseActFuture<Self, super::Result<()>>;

    fn handle(&mut self, msg: SendCommand<T>, ctx: &mut Self::Context) -> Self::Result {
        let sessions = self.sessions();
        let serialized_msg = msg.1.serialize().unwrap();

        async move {
            let sessions = sessions.lock().await;
            if let Some(session) = sessions.get(&msg.0) {
                session.recipient.do_send(Message(serialized_msg));
                return Ok(());
            }

            Err("Client not found".into())
        }
        .into_actor(self)
        .boxed_local()
    }
}

impl Handler<Disconnect> for BotServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) {
        let id = msg.0.clone();
        let sessions = self.sessions();

        async move {
            let sessions = sessions.lock().await;
            if let Some(session) = sessions.get(&id) {
                session.recipient.do_send(Message("disconnect".to_string()));
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}
