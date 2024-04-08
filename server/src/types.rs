use tokio_util::bytes::Bytes;

pub type DataUnit = Bytes;
pub type Sender = tokio::sync::mpsc::UnboundedSender<DataUnit>;
pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<DataUnit>;
