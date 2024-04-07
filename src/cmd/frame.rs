use futures::Stream;
use log::debug;
use std::io::{BufRead, Cursor};
use tokio_stream::StreamExt;
use tokio_util::bytes::{Buf, Bytes, BytesMut};

#[derive(Debug)]
pub enum FrameError {
    Incomplete,
    InvalidFrame,
    EndOfStream,
}

#[derive(Debug, Clone)]
pub enum Frame {
    Simple(String),
    Integer(i64),
    Bytes(Vec<u8>),
    Array(Vec<Frame>),
    Error(String),
}

type Result<T> = std::result::Result<T, FrameError>;

pub trait Serialize<T>: Sized {
    fn serialize(&self) -> Result<T>;
}

pub trait Deserialize<T>: Sized {
    fn deserialize(buf: T) -> Result<Self>;
}

impl Frame {
    pub fn new() -> Self {
        Frame::Array(Vec::new())
    }

    pub fn push_bytes(&mut self, bytes: &'_ [u8]) {
        match self {
            Frame::Array(array) => array.push(Frame::Bytes(bytes.to_vec())),
            _ => panic!("Cannot push bytes to non-array frame"),
        }
    }

    pub fn push_str(&mut self, s: &'_ str) {
        match self {
            Frame::Array(array) => array.push(Frame::Simple(s.to_string())),
            _ => panic!("Cannot push string to non-array frame"),
        }
    }

    pub fn push_string(&mut self, s: String) {
        match self {
            Frame::Array(array) => array.push(Frame::Simple(s)),
            _ => panic!("Cannot push string to non-array frame"),
        }
    }

    pub fn prefix(&self) -> &[u8] {
        match self {
            Frame::Simple(_) => b"+",
            Frame::Integer(_) => b":",
            Frame::Bytes(_) => b"$",
            Frame::Array(_) => b"*",
            Frame::Error(_) => b"-",
        }
    }

    pub async fn from_stream<
        T: Stream<Item = std::result::Result<BytesMut, std::io::Error>> + Unpin,
    >(
        stream: &mut T,
        buffer: &mut BytesMut,
    ) -> Result<Self> {
        loop {
            let result = stream.next().await;

            match result {
                Some(Ok(bytes)) => {
                    debug!("Received bytes: {:?}", bytes);
                    buffer.extend_from_slice(&bytes);
                }
                Some(Err(_)) => {
                    return Err(FrameError::InvalidFrame);
                }
                None => {
                    return Err(FrameError::EndOfStream);
                }
            }

            let bytes_read = buffer.clone().freeze();
            let frame = match Frame::deserialize(bytes_read) {
                Ok(f) => {
                    buffer.clear();
                    f
                }
                Err(FrameError::Incomplete) => {
                    continue;
                }
                Err(_) => {
                    return Err(FrameError::InvalidFrame);
                }
            };

            return Ok(frame);
        }
    }
}

impl Serialize<Bytes> for Frame {
    fn serialize(&self) -> Result<Bytes> {
        match self {
            Frame::Simple(s) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(self.prefix());
                buf.extend_from_slice(s.as_bytes());
                buf.extend_from_slice(b"\r\n");
                Ok(buf.freeze())
            }
            Frame::Integer(i) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(self.prefix());
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                Ok(buf.freeze())
            }
            Frame::Bytes(b) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(self.prefix());
                buf.extend_from_slice(b.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf.extend_from_slice(b);
                buf.extend_from_slice(b"\r\n");
                Ok(buf.freeze())
            }
            Frame::Array(a) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(self.prefix());
                buf.extend_from_slice(a.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                for frame in a {
                    let serialized = match frame.serialize() {
                        Ok(s) => s,
                        Err(e) => return Err(e),
                    };
                    buf.extend_from_slice(&serialized);
                }
                Ok(buf.freeze())
            }
            Frame::Error(e) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(self.prefix());
                buf.extend_from_slice(e.as_bytes());
                buf.extend_from_slice(b"\r\n");
                Ok(buf.freeze())
            }
        }
    }
}

impl Deserialize<Bytes> for Frame {
    fn deserialize(buf: Bytes) -> Result<Self> {
        let mut cursor = Cursor::new(buf);

        if cursor.remaining() == 0 {
            return Err(FrameError::Incomplete);
        }

        match cursor.get_u8() {
            b'+' => {
                let mut line = String::new();
                match cursor.read_line(&mut line) {
                    Ok(_) => {}
                    Err(_) => return Err(FrameError::InvalidFrame),
                }

                line = line.trim().to_string();
                Ok(Frame::Simple(line))
            }
            b':' => {
                let mut line = String::new();
                match cursor.read_line(&mut line) {
                    Ok(_) => {}
                    Err(_) => return Err(FrameError::InvalidFrame),
                }

                line = line.trim().to_string();
                match line.parse() {
                    Ok(n) => Ok(Frame::Integer(n)),
                    Err(_) => return Err(FrameError::InvalidFrame),
                }
            }
            b'$' => {
                let mut line = String::new();
                cursor.read_line(&mut line).unwrap();
                line = line.trim().to_string();
                let len = line.parse().unwrap();
                let mut bytes = vec![0; len];
                cursor.copy_to_slice(&mut bytes);
                Ok(Frame::Bytes(bytes))
            }
            b'*' => {
                let mut line = String::new();
                match cursor.read_line(&mut line) {
                    Ok(_) => {}
                    Err(_) => return Err(FrameError::InvalidFrame),
                }

                line = line.trim().to_string();
                let len = match line.parse() {
                    Ok(n) => n,
                    Err(_) => return Err(FrameError::InvalidFrame),
                };
                debug!("Array length: {}", len);

                let mut array = Vec::with_capacity(len);
                for _ in 0..len {
                    let mut line = String::new();
                    match cursor.read_line(&mut line) {
                        Ok(_) => {}
                        Err(_) => return Err(FrameError::InvalidFrame),
                    }

                    line = line.trim().to_string();

                    let frame = match Frame::deserialize(Bytes::from(line)) {
                        Ok(f) => f,
                        Err(e) => return Err(e),
                    };

                    array.push(frame);
                }
                Ok(Frame::Array(array))
            }
            b'-' => {
                let mut line = String::new();
                cursor.read_line(&mut line).unwrap();
                line = line.trim().to_string();
                Ok(Frame::Error(line))
            }
            _ => Err(FrameError::InvalidFrame),
        }
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Frame::Simple(a), Frame::Simple(b)) => a == b,
            (Frame::Integer(a), Frame::Integer(b)) => a == b,
            (Frame::Bytes(a), Frame::Bytes(b)) => a == b,
            (Frame::Array(a), Frame::Array(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(a) => a.eq(other),
            Frame::Bytes(a) => a.to_vec().eq(other.as_bytes()),
            _ => false,
        }
    }
}
