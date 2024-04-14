mod websocket;

use actix::Actor;
use websocket::client::Client;

const BACKOFF_MAX_DURATION: u16 = 2048;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_endpoint = "http://127.0.0.1:8080/api/sessions/ws";

    let client = Client::new(server_endpoint);
    let _ = client.start();

    tokio::signal::ctrl_c().await?;

    Ok(())
}
