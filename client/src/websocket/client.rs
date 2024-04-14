use std::borrow::BorrowMut;

use actix::{io::SinkWrite, Actor, ActorContext, Addr, AsyncContext, Context, StreamHandler};
use actix_codec::Framed;
use awc::BoxedSocket;
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};

use super::session::Session;

type WebSocketStream = SplitStream<Framed<BoxedSocket, awc::ws::Codec>>;
type WebSocketWriter = SplitSink<Framed<BoxedSocket, awc::ws::Codec>, awc::ws::Message>;
type StreamResult<T> = std::result::Result<T, awc::error::WsProtocolError>;

#[derive(Debug)]
pub enum ClientError {
    ConnectionError,
    AlreadyConnected,
    WebSocketClientError(awc::error::WsClientError),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClientError::ConnectionError => write!(f, "Error connecting to server"),
            ClientError::AlreadyConnected => write!(f, "Client already connected"),
            ClientError::WebSocketClientError(e) => write!(f, "WebSocket client error: {:?}", e),
        }
    }
}

impl From<awc::error::WsClientError> for ClientError {
    fn from(e: awc::error::WsClientError) -> Self {
        ClientError::WebSocketClientError(e)
    }
}

impl std::error::Error for ClientError {}

type Result<T> = std::result::Result<T, ClientError>;

pub struct Client {
    url: String,
    session: Option<Addr<Session>>,
}

impl Client {
    pub fn new(url: &str) -> Self {
        Client {
            url: url.to_string(),
            session: None,
        }
    }

    pub async fn start_session(&mut self) -> Result<()> {
        let url = self.url.clone();
        let client = awc::Client::new();
        let (_, ws) = client.ws(url.clone()).connect().await?;
        let (writer, reader) = ws.split();

        self.session = Some(Session::connect(writer, reader));

        Ok(())
    }

    pub fn connected(&mut self) -> bool {
        if let Some(session) = self.session.borrow_mut() {
            return session.connected();
        }

        false
    }
}
