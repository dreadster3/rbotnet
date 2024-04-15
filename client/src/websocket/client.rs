use std::borrow::BorrowMut;

use actix::{
    dev::ContextFutureSpawner, io::SinkWrite, Actor, ActorContext, ActorFutureExt, Addr,
    AsyncContext, Context, Handler, ResponseActFuture, StreamHandler, WrapFuture,
};
use actix_codec::Framed;
use awc::BoxedSocket;
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::debug;
use protocol::commands::{command::Command, Deserialize};

use super::{events::Connect, session::Session};

type WebSocketStream = SplitStream<Framed<BoxedSocket, awc::ws::Codec>>;
type WebSocketWriter = SplitSink<Framed<BoxedSocket, awc::ws::Codec>, awc::ws::Message>;
type StreamResult<T> = std::result::Result<T, awc::error::WsProtocolError>;

#[derive(Debug)]
pub enum ClientError {
    ConnectionError,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClientError::ConnectionError => write!(f, "Error connecting to server"),
        }
    }
}

impl std::error::Error for ClientError {}

type Result<T> = std::result::Result<T, ClientError>;

const BACKOFF_MAX_DURATION: u16 = 2048;

pub struct Client {
    url: String,
    pub writer: Option<WebSocketWriter>,
}

impl Client {
    pub fn new(url: &str) -> Self {
        Client {
            url: url.to_string(),
            writer: None,
        }
    }

    async fn connect(url: &str) -> Result<(WebSocketWriter, WebSocketStream)> {
        let (_, ws) = awc::Client::new()
            .ws(url)
            .connect()
            .await
            .map_err(|_| ClientError::ConnectionError)?;

        println!("Connected to server");

        Ok(ws.split())
    }

    async fn connect_retry(url: &str) -> (WebSocketWriter, WebSocketStream) {
        let mut backoff_duration = 1u16;

        loop {
            match Self::connect(url).await {
                Ok(ws) => return ws,
                Err(e) => {
                    debug!("Error connecting to server: {:?}", e);
                    println!(
                        "Error connecting to server! Backing off for {} seconds",
                        backoff_duration
                    );
                    backoff_duration = (backoff_duration * 2).clamp(0, BACKOFF_MAX_DURATION);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(backoff_duration as u64)).await;
        }
    }
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Client started! Connecting to server...");
        ctx.address().do_send(Connect {});
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("Client stopped");
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::prelude::Running {
        println!("Client disconnected! Reconnecting...");
        ctx.address().do_send(Connect {});
        actix::prelude::Running::Continue
    }
}

impl Handler<Connect> for Client {
    type Result = ();

    fn handle(&mut self, _: Connect, ctx: &mut Self::Context) -> Self::Result {
        let url = self.url.clone();

        async move { Self::connect_retry(&url.clone()).await }
            .into_actor(self)
            .then(|result, actor, ctx| {
                let (writer, stream) = result;
                actor.writer = Some(writer);
                ctx.add_stream(stream);

                actix::fut::ready(())
            })
            .wait(ctx);
    }
}

impl StreamHandler<StreamResult<awc::ws::Frame>> for Client {
    fn handle(&mut self, msg: StreamResult<awc::ws::Frame>, ctx: &mut Self::Context) {
        match msg {
            Ok(awc::ws::Frame::Text(text)) => {
                let message = String::from_utf8_lossy(&text);
                debug!("Received text frame: {:?}", text);

                let command = match Command::deserialize(message.to_string()) {
                    Ok(command) => command,
                    Err(e) => {
                        eprintln!("Error executing command: {:?}", e);
                        return;
                    }
                };

                async move {
                    match command.execute().await {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error executing command: {:?}", e),
                    }
                }
                .into_actor(self)
                .spawn(ctx);
            }
            Ok(awc::ws::Frame::Binary(bin)) => {
                debug!("Received binary frame: {:?}", bin);
            }
            Ok(awc::ws::Frame::Ping(_)) => {
                debug!("Received ping frame");
            }
            Ok(awc::ws::Frame::Pong(_)) => {
                debug!("Received pong frame");
            }
            Ok(awc::ws::Frame::Continuation(item)) => {
                debug!("Received continuation frame: {:?}", item);
            }
            Ok(awc::ws::Frame::Close(reason)) => {
                debug!("Received close frame: {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                debug!("Error receiving frame: {:?}", e);
                ctx.stop();
            }
        }
    }
}
