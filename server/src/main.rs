use context::{known_keys, ContextDataType};
use log::MultiWriter;
use server::Server;
use structured_logger::{async_json, Builder};
use tokio::{fs::OpenOptions, io::stdout};

pub mod connection;
pub mod context;
pub mod log;
pub mod server;
pub mod session;
pub mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "127.0.0.1";
    let port = 8080;

    let mut open_options = OpenOptions::new();
    open_options
        .create(true)
        .truncate(false)
        .read(false)
        .write(true)
        .append(true);

    // Write to multiple destinations
    let default = MultiWriter::new(vec![
        async_json::new_writer(open_options.open("logs/server.log").await?),
        async_json::new_writer(stdout()),
    ]);

    Builder::new().with_default_writer(Box::new(default)).init();

    let client_server = Server::new(host, port).await.unwrap();
    let clinet_connections = client_server.get_connections();
    let admin_server = Server::new(host, port + 1).await.unwrap();
    admin_server
        .set_to_context(
            known_keys::CONNECTIONS_KEY,
            ContextDataType::from(clinet_connections),
        )
        .await;

    tokio::try_join!(client_server.run(), admin_server.run())?;

    return Ok(());
}
