use std::fs::OpenOptions;
use std::io::stdout;

use rbotnet::{log::multi_writer::MultiWriter, server::server::Server};
use structured_logger::{json, Builder};

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
    let writer = MultiWriter::new(vec![
        json::new_writer(open_options.open("logs/server.log").unwrap()),
        json::new_writer(stdout()),
    ]);

    Builder::new()
        .with_target_writer("*", Box::new(writer))
        .init();

    let host = "127.0.0.1";
    let port = 8080;

    let mut server = Server::new(host, port).await?;

    server.run().await?;

    return Ok(());
}
