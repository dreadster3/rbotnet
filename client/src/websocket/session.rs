use actix::{io::SinkWrite, Actor, ActorContext, Addr, AsyncContext, Context, StreamHandler};
use log::debug;

use actix_codec::Framed;
use awc::BoxedSocket;
use futures_util::stream::{SplitSink, SplitStream};

type WebSocketStream = SplitStream<Framed<BoxedSocket, awc::ws::Codec>>;
type WebSocketWriter = SplitSink<Framed<BoxedSocket, awc::ws::Codec>, awc::ws::Message>;
type StreamResult<T> = std::result::Result<T, awc::error::WsProtocolError>;

pub struct Session {
    writer: SinkWrite<awc::ws::Message, WebSocketWriter>,
}

impl Session {
    pub fn connect(writer: WebSocketWriter, stream: WebSocketStream) -> Addr<Self> {
        let addr = Self::create(|ctx| {
            ctx.add_stream(stream);
            Session {
                writer: SinkWrite::new(writer, ctx),
            }
        });

        addr
    }
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Client connected");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Client disconnected");
    }
}

impl actix::io::WriteHandler<awc::error::WsProtocolError> for Session {}

impl StreamHandler<StreamResult<awc::ws::Frame>> for Session {
    fn handle(&mut self, msg: StreamResult<awc::ws::Frame>, ctx: &mut Self::Context) {
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
