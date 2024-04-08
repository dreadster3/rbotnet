use std::{net::SocketAddr, sync::Arc};

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_util::{
    bytes::BytesMut,
    codec::{BytesCodec, Framed},
};

use crate::{
    connection::Connection,
    context::Context,
    types::{DataUnit, Receiver, Sender},
};

pub struct Session {
    id: String,
    address: SocketAddr,

    channel_receiver: Receiver,
    channel_transmitter: Sender,
    stream: Framed<TcpStream, BytesCodec>,
    stream_buffer: BytesMut,
}

#[derive(Debug)]
pub enum SessionError {
    SendError(tokio::sync::mpsc::error::SendError<DataUnit>),
    IOError(std::io::Error),
    EndOfStream,
}

type Result<T> = std::result::Result<T, SessionError>;

impl Session {
    pub fn new(stream: TcpStream) -> Result<(Self, Connection)> {
        let id = uuid::Uuid::new_v4().to_string();
        let address = stream.peer_addr()?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<DataUnit>();
        let connection = Connection::new(id.clone(), address, tx.clone());

        let obj = Self {
            id,
            address,
            channel_receiver: rx,
            channel_transmitter: tx,
            stream: Framed::new(stream, BytesCodec::new()),
            stream_buffer: BytesMut::new(),
        };

        Ok((obj, connection))
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_connection(&self) -> Connection {
        Connection::new(
            self.id.to_string(),
            self.address.to_owned(),
            self.channel_transmitter.clone(),
        )
    }

    pub fn get_channel(&self) -> &Receiver {
        &self.channel_receiver
    }

    pub fn get_stream(&self) -> &Framed<TcpStream, BytesCodec> {
        &self.stream
    }

    pub fn get_stream_buffer(&self) -> &BytesMut {
        &self.stream_buffer
    }

    pub async fn handle(&mut self, _: Arc<Mutex<Context>>) -> Result<()> {
        loop {
            tokio::select! {
                Some(data) = self.channel_receiver.recv() => {
                    self.stream.send(data).await?;
                }
                result = self.stream.next() => {
                    match result {
                        Some(Ok(data)) => {
                            self.channel_transmitter.send(data.freeze())?;
                        }
                        Some(Err(e)) => {
                            eprintln!("Error: {:?}", e);
                        }
                        None => {
                            return Err(SessionError::EndOfStream);
                        }
                    }
                }
            }
        }
    }
}

impl From<std::io::Error> for SessionError {
    fn from(e: std::io::Error) -> Self {
        SessionError::IOError(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<DataUnit>> for SessionError {
    fn from(e: tokio::sync::mpsc::error::SendError<DataUnit>) -> Self {
        SessionError::SendError(e)
    }
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::SendError(e) => write!(f, "SendError: {}", e),
            SessionError::IOError(e) => write!(f, "IOError: {}", e),
            SessionError::EndOfStream => write!(f, "EndOfStream"),
        }
    }
}
