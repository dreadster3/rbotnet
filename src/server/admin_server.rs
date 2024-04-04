use std::sync::Arc;

use log::info;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};

use crate::server::admin_session::AdminSession;

use super::state::{self, State};

pub struct AdminServer {
    listener: TcpListener,

    state: State,
    client_state: State,

    limit_connections: Arc<Semaphore>,
}

impl AdminServer {
    pub async fn new(client_state: State, host: &str, port: u16) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

        Ok(Self {
            listener,
            client_state: Arc::clone(&client_state),
            state: state::new(),
            limit_connections: Arc::new(Semaphore::new(1000)),
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        info!(target: "admin_server_events", "Accepting connections on: {}", self.listener.local_addr()?);

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;
            let state = Arc::clone(&self.state);
            let (mut session, transmitter) = AdminSession::new(socket).await?;

            state::add_session(Arc::clone(&state), session.get_id(), transmitter).await;

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
                    log::info!(target: "admin_server_events", "Accepted connection from: {}", socket.peer_addr()?);
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
