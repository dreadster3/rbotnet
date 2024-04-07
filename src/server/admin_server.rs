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

    admin_connections: State,
    client_connections: State,

    limit_connections: Arc<Semaphore>,
}

impl AdminServer {
    pub async fn new(client_state: State, host: &str, port: u16) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

        Ok(Self {
            listener,
            client_connections: Arc::clone(&client_state),
            admin_connections: state::new(),
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
            let admin_connections = Arc::clone(&self.admin_connections);
            let (mut admin_session, transmitter) = AdminSession::new(socket).await?;

            let connection = state::Connection::new(
                admin_session.get_id().to_string(),
                admin_session.get_address().to_owned(),
                transmitter,
            );

            state::add_connection(Arc::clone(&admin_connections), connection).await;

            let client_connections = Arc::clone(&self.client_connections);
            tokio::spawn(async move {
                admin_session
                    .handle(admin_connections, client_connections)
                    .await
                    .unwrap();
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
