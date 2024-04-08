use std::{collections::HashMap, sync::Arc};

use log::{debug, error, info};
use tokio::sync::{Mutex, Semaphore};

use crate::{
    connection::Connection,
    context::{known_keys, Context, ContextDataType},
    session::{Session, SessionError},
};

#[derive(Debug)]
pub enum ServerError {
    SessionError(SessionError),
    IOError(std::io::Error),
}

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
pub struct Server {
    listener: tokio::net::TcpListener,
    semaphore: Arc<Semaphore>,
    connections: Arc<Mutex<HashMap<String, Connection>>>,

    context: Arc<Mutex<Context>>,
}

impl Server {
    pub async fn new(host: &str, port: u16) -> Result<Server> {
        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        Ok(Server {
            listener,
            semaphore: Arc::new(Semaphore::new(1000)),
            connections: Arc::new(Mutex::new(HashMap::new())),

            context: Arc::new(Mutex::new(Context::new())),
        })
    }

    pub fn get_connections(&self) -> Arc<Mutex<HashMap<String, Connection>>> {
        Arc::clone(&self.connections)
    }

    pub fn get_context(&self) -> Arc<Mutex<Context>> {
        Arc::clone(&self.context)
    }

    pub async fn get_from_context(&self, key: &str) -> Option<ContextDataType> {
        let context = self.context.lock().await;
        context.get(key).cloned()
    }

    pub async fn set_to_context(&self, key: &str, value: ContextDataType) {
        let mut context = self.context.lock().await;
        context.insert(key.to_string(), value);
    }

    pub async fn run(self) -> Result<()> {
        info!(target: "server_events", "Accepting connections on: {}", self.listener.local_addr()?);

        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let socket = self.accept().await?;

            let (mut session, connection) = Session::new(socket).unwrap();
            let connections = Arc::clone(&self.connections);
            {
                let mut connections = connections.lock().await;
                connections.insert(connection.get_id().to_string(), connection);
            }

            let context = Arc::clone(&self.context);
            {
                let mut context = context.lock().await;
                context.insert(
                    known_keys::CONNECTIONS_KEY.to_string(),
                    ContextDataType::Connections(self.connections.clone()),
                );
            }
            tokio::spawn(async move {
                let result = match session.handle(context).await {
                    Err(SessionError::EndOfStream) => {
                        info!(target: "server_events", session_id=session.get_id(); "Received end of stream");
                        Ok(())
                    }
                    Err(e) => {
                        error!(target: "server_events", session_id=session.get_id(); "Error handling session: {}", e);
                        Err(e)
                    }
                    _ => Ok(()),
                };

                {
                    let mut connections = connections.lock().await;
                    connections.remove(session.get_id());
                }
                drop(permit);

                result
            });
        }
    }

    async fn accept(&self) -> Result<tokio::net::TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    return Ok(socket);
                }
                Err(e) => {
                    info!(target: "server_events", "Error accepting connection: {}", e);
                    if backoff > 16 {
                        return Err(ServerError::IOError(e));
                    }
                }
            }

            debug!(target: "server_events", "Backing off for {} seconds", backoff);
            tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(e: std::io::Error) -> Self {
        ServerError::IOError(e)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServerError::SessionError(e) => write!(f, "SessionError: {}", e),
            ServerError::IOError(e) => write!(f, "IOError: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}
