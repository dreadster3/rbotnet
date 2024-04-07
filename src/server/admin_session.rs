use futures::SinkExt;
use log::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_util::{
    bytes::{Bytes, BytesMut},
    codec::{BytesCodec, Framed},
};

use crate::cmd::{
    command::Command,
    frame::{Frame, FrameError},
};

use super::{state::State, Receiver, Sender};

#[derive(Debug)]
pub struct AdminSession {
    id: String,
    address: SocketAddr,
    bytes_stream: Framed<TcpStream, BytesCodec>,
    receiver: Receiver,
    buffer: BytesMut,
}

impl AdminSession {
    pub async fn new(stream: TcpStream) -> Result<(Self, Sender), std::io::Error> {
        let address = stream.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Bytes>();

        Ok((
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                address,
                bytes_stream: Framed::new(stream, BytesCodec::new()),
                receiver: rx,
                buffer: BytesMut::new(),
            },
            tx,
        ))
    }

    pub async fn handle(
        &mut self,
        admin_connections: State,
        client_connections: State,
    ) -> Result<(), std::io::Error> {
        let client_connections = Arc::clone(&client_connections);
        info!(target: "session_events", session_id=self.get_id(), address=self.get_address().to_string(); "Opening session");

        // let text = "*2\r\n+GET\r\n+SESSIONS\r\n";
        // let bytes = Bytes::from(text);
        // let frame = Frame::deserialize(bytes).unwrap();
        // self.bytes_stream.send(frame.serialize().unwrap()).await?;

        loop {
            tokio::select! {
                Some(bytes) = self.receiver.recv() => {
                    info!(target: "session_events", session_id=self.get_id(); "Sending Message: {}", String::from_utf8(bytes.clone().to_vec()).unwrap());
                    self.bytes_stream.send(bytes).await?;
                }
                result = Frame::from_stream(&mut self.bytes_stream, &mut self.buffer) => {
                    match result {
                        Ok(frame) => {
                            info!(target: "admin_session_events", session_id=self.get_id(); "Received frame: {:?}", frame);
                            let command = Command::from_frame(frame).unwrap();
                            let admin_connections = admin_connections.lock().await;
                            let admin_connection = admin_connections.get(&self.id).unwrap();

                            match command.execute(admin_connection, client_connections.clone()).await {
                                Ok(_) => {
                                    info!(target: "admin_session_events", session_id=self.get_id(); "Command executed");
                                }
                                Err(e) => {
                                    error!(target: "admin_session_events", session_id=self.get_id(); "Error executing command: {}", e);
                                }
                            }

                        }
                        Err(FrameError::EndOfStream) => {
                            info!(target: "admin_session_events", session_id=self.get_id(); "End of stream");
                            break;
                        }
                        Err(e) => {
                            error!(target: "admin_session_events", session_id=self.get_id(); "Error reading line: {:?}", e);
                        }
                    }
                }
            }
        }

        info!(target: "session_events", session_id=self.get_id(), address=self.get_address().to_string(); "Closing session");

        {
            let mut sessions = client_connections.lock().await;
            sessions.remove(&self.id);
            info!(target: "server_events", session_id=self.get_id(); "Session removed from state");
        }

        Ok(())
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }
}
