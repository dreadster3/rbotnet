use rbotnet::{
    log::multi_writer::MultiWriter,
    server::{admin_server::AdminServer, server::Server},
};
use structured_logger::{async_json, Builder};
use tokio::fs::OpenOptions;
use tokio::io::stdout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let admin = MultiWriter::new(vec![
        async_json::new_writer(open_options.open("logs/admin_server.log").await?),
        async_json::new_writer(stdout()),
    ]);

    Builder::new()
        .with_default_writer(Box::new(default))
        .with_target_writer("admin*", Box::new(admin))
        .init();

    let host = "127.0.0.1";
    let port = 8080;

    let server = Server::new(host, port).await?;
    let server_state = server.get_state().await;
    let admin_server = AdminServer::new(server_state, host, port + 1).await?;

    tokio::try_join!(server.run(), admin_server.run())?;

    return Ok(());
}
