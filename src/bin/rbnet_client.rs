use futures::{SinkExt, StreamExt};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "127.0.0.1";
    let port = 8080;
    let addr = format!("{}:{}", host, port);

    println!("Connecting to: {}", addr);

    let stream = TcpStream::connect(&addr).await?;

    let mut framed_stream = Framed::new(stream, LinesCodec::new());

    loop {
        let msg = "Hello, world!";

        framed_stream.send(msg).await?;

        println!("Sent: {}", msg);

        match framed_stream.next().await {
            Some(Ok(line)) => {
                println!("Received: {}", line);
            }
            Some(Err(e)) => {
                eprintln!("Error: {}", e);
            }
            None => break,
        }
    }

    Ok(())
}
