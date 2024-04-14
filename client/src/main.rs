mod websocket;

use log::debug;
use websocket::client::Client;

const BACKOFF_MAX_DURATION: u16 = 2048;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_endpoint = "http://127.0.0.1:8080/api/sessions/ws";

    let mut backoff_duration = 1u16;
    let mut client = Client::new(server_endpoint);

    loop {
        if client.connected() {
            debug!("Client still connected!");
            tokio::time::sleep(tokio::time::Duration::from_secs(60 * 5)).await;
            continue;
        }

        println!("Client disconnected, reconnecting...");

        match client.start_session().await {
            Ok(_) => {
                backoff_duration = 1;
                continue;
            }
            Err(e) => {
                eprintln!("Error starting session: {}", e);
                backoff_duration = backoff_duration * 2;
            }
        }

        backoff_duration = backoff_duration.clamp(1, BACKOFF_MAX_DURATION);
        println!("Retrying in {} seconds...", backoff_duration);
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_duration as u64)).await;
    }
}
