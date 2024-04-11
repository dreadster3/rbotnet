use std::net::SocketAddr;

use actix::{
    dev::ContextFutureSpawner, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler,
    StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use tokio::sync::OwnedSemaphorePermit;

use super::{
    messages::{Connected, Disconnected, Message},
    server::BotServer,
};

pub struct BotSession {
    id: String,
    address: SocketAddr,

    _permit: OwnedSemaphorePermit,
    server: Addr<BotServer>,
}

impl BotSession {
    pub fn new(server: Addr<BotServer>, address: SocketAddr, permit: OwnedSemaphorePermit) -> Self {
        return Self {
            id: String::new(),
            address,
            server,
            _permit: permit,
        };
    }
}

impl Handler<Message> for BotSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Actor for BotSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();

        let connected = Connected::new(self.address, addr.recipient());

        self.server
            .send(connected)
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(Ok(id)) => {
                        act.id = id;
                    }
                    _ => ctx.stop(),
                }

                actix::fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::prelude::Running {
        let disconnection = Disconnected::new(self.id.clone());

        self.server.do_send(disconnection);

        actix::prelude::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for BotSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(_)) => {
                ctx.text("Server does not accept messages");
            }
            Ok(ws::Message::Binary(_)) => {
                ctx.text("Server does not accept messages");
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.text("Server does not accept messages");
            }
            _ => ctx.stop(),
        }
    }
}
