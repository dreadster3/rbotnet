use std::{
    fmt::{self, Formatter},
    vec,
};

use super::frame::Frame;

pub struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum ParseError {
    Invalid,
    EndOfStream,
    Other,
}

type Result<T> = std::result::Result<T, ParseError>;

impl Parse {
    pub fn new(frame: Frame) -> Result<Self> {
        match frame {
            Frame::Array(frames) => Ok(Self {
                parts: frames.into_iter(),
            }),
            _ => Err(ParseError::Invalid),
        }
    }

    pub fn next(&mut self) -> Result<Frame> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bytes(b) => Ok(String::from_utf8(b.to_vec()).map_err(|_| ParseError::Invalid)?),
            _ => Err(ParseError::Invalid),
        }
    }

    pub fn next_bytes(&mut self) -> Result<Vec<u8>> {
        match self.next()? {
            Frame::Simple(s) => Ok(s.into_bytes()),
            Frame::Bytes(b) => Ok(b.to_vec()),
            _ => Err(ParseError::Invalid),
        }
    }

    pub fn next_integer(&mut self) -> Result<i64> {
        match self.next()? {
            Frame::Integer(i) => Ok(i),
            Frame::Simple(s) => s.parse().map_err(|_| ParseError::Invalid),
            Frame::Bytes(b) => String::from_utf8(b.to_vec())
                .map_err(|_| ParseError::Invalid)
                .and_then(|s| s.parse().map_err(|_| ParseError::Invalid)),
            _ => Err(ParseError::Invalid),
        }
    }
}

impl From<String> for ParseError {
    fn from(_: String) -> ParseError {
        ParseError::Other
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        src.to_string().into()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParseError::Invalid => "protocol error; invalid frame".fmt(f),
            ParseError::Other => "protocol error".fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
