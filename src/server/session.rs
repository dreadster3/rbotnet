use futures::SinkExt;
use log::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::bytes::Bytes;
use tokio_util::codec::{BytesCodec, Framed};

use super::state::State;
use super::{Receiver, Sender};

#[derive(Debug)]
pub struct Session {
    id: String,
    address: SocketAddr,
    line_framed_stream: Framed<TcpStream, BytesCodec>,
    receiver: Receiver,
}

impl Session {
    pub async fn new(stream: TcpStream) -> Result<(Self, Sender), std::io::Error> {
        let address = stream.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Bytes>();

        Ok((
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                address,
                line_framed_stream: Framed::new(stream, BytesCodec::new()),
                receiver: rx,
            },
            tx,
        ))
    }

    pub async fn handle(&mut self, state: State) -> Result<(), std::io::Error> {
        let state = Arc::clone(&state);
        info!(target: "session_events", session_id=self.get_id(), address=self.get_address().to_string(); "Opening session");
        loop {
            tokio::select! {
                Some(bytes) = self.receiver.recv() => {
                    let msg = String::from_utf8(bytes.to_vec()).unwrap();
                    info!(target: "session_events", session_id=self.get_id(); "Sending Message: {}", msg);
                    match self.line_framed_stream.send(Bytes::from(format!("Server: {}", msg))).await {
                        Ok(_) => {
                            info!(target: "session_events", session_id=self.get_id(); "Message sent");
                        }
                        Err(e) => {
                            error!(target: "session_events", session_id=self.get_id(); "Error sending message: {}", e);
                        }
                    }
                }
                result = self.line_framed_stream.next() => {
                    match result {
                        Some(Ok(bytes)) => {
                            let line = String::from_utf8(bytes.to_vec()).unwrap();
                            info!(target: "session_events", session_id=self.get_id(); "Received: {}", line);
                            let sessions = state.lock().await;
                            let connection = sessions.get(&self.id).unwrap();
                            let transmitter = connection.get_sender();
                            transmitter.send(Bytes::from(line)).unwrap();
                        }
                        Some(Err(e)) => {
                            error!(target: "session_events", session_id=self.get_id(); "Error reading line: {}", e);
                        }
                        None => break
                    }
                }
            }
        }

        info!(target: "session_events", session_id=self.get_id(), address=self.get_address().to_string(); "Closing session");

        {
            let mut sessions = state.lock().await;
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
