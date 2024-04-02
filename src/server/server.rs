use log::info;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{Mutex, Semaphore},
};

use crate::server::session::Session;

pub struct Server {
    listener: tokio::net::TcpListener,

    limit_connections: Arc<Semaphore>,

    sessions:
        Arc<Mutex<std::collections::HashMap<String, tokio::sync::mpsc::UnboundedSender<String>>>>,
}

impl Server {
    pub async fn new(host: &str, port: u16) -> Result<Server, std::io::Error> {
        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        Ok(Server {
            listener,
            limit_connections: Arc::new(Semaphore::new(1000)),
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
    }

    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        info!(target: "server_events", "Accepting connections on: {}", self.listener.local_addr()?);

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;

            let (mut session, transmitter) = Session::new(socket).await?;
            let session_id = session.get_id();

            let connections = Arc::clone(&self.sessions);

            {
                let mut sessions = connections.lock().await;
                sessions.insert(session_id.to_string(), transmitter);
            }

            tokio::spawn(async move {
                session.handle(connections).await.unwrap();
                drop(permit);
            });
        }
    }

    pub async fn accept(&self) -> Result<TcpStream, tokio::io::Error> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    log::info!(target: "server_events", "Accepted connection from: {}", socket.peer_addr()?);
                    return Ok(socket);
                }
                Err(e) => {
                    if backoff > 16 {
                        return Err(e);
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}
