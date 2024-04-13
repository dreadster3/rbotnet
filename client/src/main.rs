use actix::{io::SinkWrite, Actor, ActorContext, Addr, AsyncContext, Context, StreamHandler};
use actix_codec::Framed;
use awc::{http, BoxedSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::debug;

type WebSocketStream = SplitStream<Framed<BoxedSocket, awc::ws::Codec>>;
type WebSocketWriter = SplitSink<Framed<BoxedSocket, awc::ws::Codec>, awc::ws::Message>;

struct WsClient {
    writer: SinkWrite<awc::ws::Message, WebSocketWriter>,
}

impl WsClient {
    pub fn start(writer: WebSocketWriter, stream: WebSocketStream) -> Addr<Self> {
        WsClient::create(|ctx| {
            ctx.add_stream(stream);
            WsClient {
                writer: SinkWrite::new(writer, ctx),
            }
        })
    }
}

impl Actor for WsClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Client connected");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Client disconnected");
    }
}

impl actix::io::WriteHandler<awc::error::WsProtocolError> for WsClient {}

impl StreamHandler<Result<awc::ws::Frame, awc::error::WsProtocolError>> for WsClient {
    fn handle(
        &mut self,
        msg: Result<awc::ws::Frame, awc::error::WsProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(awc::ws::Frame::Text(text)) => {
                println!("Received text frame: {:?}", String::from_utf8_lossy(&text));
                debug!("Received text frame: {:?}", text);
            }
            Ok(awc::ws::Frame::Binary(bin)) => {
                debug!("Received binary frame: {:?}", bin);
            }
            Ok(awc::ws::Frame::Ping(_)) => {
                debug!("Received ping frame");
            }
            Ok(awc::ws::Frame::Pong(_)) => {
                debug!("Received pong frame");
            }
            Ok(awc::ws::Frame::Continuation(item)) => {
                debug!("Received continuation frame: {:?}", item);
            }
            Ok(awc::ws::Frame::Close(reason)) => {
                debug!("Received close frame: {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                debug!("Error receiving frame: {:?}", e);
                ctx.stop();
            }
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_endpoint = "http://127.0.0.1:8080/api/sessions/ws";

    let (_, ws) = match awc::Client::new().ws(server_endpoint).connect().await {
        Ok(res) => res,
        Err(e) => {
            println!("Error connecting to server: {:?}", e);
            return Ok(());
        }
    };

    let (writer, reader) = ws.split();

    let mut client = WsClient::start(writer, reader);

    loop {
        if !client.connected() {
            println!("Client disconnected, reconnecting...");

            let (_, ws) = match awc::Client::new().ws(server_endpoint).connect().await {
                Ok(res) => res,
                Err(e) => {
                    println!("Error connecting to server: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    continue;
                }
            };

            let (writer, reader) = ws.split();

            client = WsClient::start(writer, reader);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
