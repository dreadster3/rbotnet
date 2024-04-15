use protocol::commands::{Deserialize, Serialize};

use super::server::BotConnection;
use super::Result;
use std::net::SocketAddr;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(actix::Message)]
#[rtype(result = "Result<String>")]
pub struct Connected {
    pub id: String,
    pub address: SocketAddr,

    pub recipient: actix::Recipient<Message>,
}

impl Connected {
    pub fn new(address: SocketAddr, recipient: actix::Recipient<Message>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        return Self {
            id,
            address,
            recipient,
        };
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Disconnected(pub String);

#[derive(actix::Message)]
#[rtype(result = "Vec<BotConnection>")]
pub struct ListSessions;

#[derive(actix::Message)]
#[rtype(result = "Result<()>")]
pub struct BroadcastCommand<T>(pub T)
where
    T: Serialize + Deserialize;

#[derive(actix::Message)]
#[rtype(result = "Result<()>")]
pub struct SendCommand<T>(pub String, pub T)
where
    T: Serialize + Deserialize;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct Disconnect(pub String);
