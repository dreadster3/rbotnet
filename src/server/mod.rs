use tokio_util::bytes::Bytes;

pub mod server;
pub mod session;

pub mod admin_server;
pub mod admin_session;

pub mod state;

pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<Bytes>;
pub type Sender = tokio::sync::mpsc::UnboundedSender<Bytes>;
