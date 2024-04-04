use futures::SinkExt;
use log::{error, info};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use super::state::State;

#[derive(Debug)]
pub struct AdminSession {
    id: String,
    address: SocketAddr,
    line_framed_stream: Framed<TcpStream, LinesCodec>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<String>,
}

impl AdminSession {
    pub async fn new(
        stream: TcpStream,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedSender<String>), std::io::Error> {
        let address = stream.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        Ok((
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                address,
                line_framed_stream: Framed::new(stream, LinesCodec::new()),
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
                Some(msg) = self.receiver.recv() => {
                    info!(target: "session_events", session_id=self.get_id(); "Sending Message: {}", msg);
                    match self.line_framed_stream.send(format!("Server: {}", msg)).await {
                        Ok(_) => {
                            info!(target: "admin_session_events", session_id=self.get_id(); "Message sent");
                        }
                        Err(e) => {
                            error!(target: "admin_session_events", session_id=self.get_id(); "Error sending message: {}", e);
                        }
                    }
                }
                result = self.line_framed_stream.next() => {
                    match result {
                        Some(Ok(line)) => {
                            info!(target: "admin_session_events", session_id=self.get_id(); "Received: {}", line);
                            let sessions = state.lock().await;
                            let transmitter = sessions.get(&self.id).unwrap();
                            transmitter.send(line).unwrap();
                        }
                        Some(Err(e)) => {
                            error!(target: "admin_session_events", session_id=self.get_id(); "Error reading line: {}", e);
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
