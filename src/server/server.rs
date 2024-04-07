use log::info;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Semaphore};

use crate::server::session::Session;
use crate::server::state::{Connection, State};

use super::state;

pub struct Server {
    listener: tokio::net::TcpListener,

    limit_connections: Arc<Semaphore>,

    state: State,
}

impl Server {
    pub async fn new(host: &str, port: u16) -> Result<Server, std::io::Error> {
        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        Ok(Server {
            listener,
            limit_connections: Arc::new(Semaphore::new(1000)),
            state: state::new(),
        })
    }

    pub async fn get_state(&self) -> State {
        Arc::clone(&self.state)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
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
            let state = Arc::clone(&self.state);
            let connection = Connection::new(
                session_id.to_string(),
                session.get_address().to_owned(),
                transmitter,
            );

            state::add_connection(Arc::clone(&state), connection).await;

            tokio::spawn(async move {
                session.handle(state).await.unwrap();
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
