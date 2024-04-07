mod sessions;

use crate::cmd::{frame::Frame, parse::Parse};

use self::sessions::GetSessions;

pub enum Get {
    Sessions(GetSessions),
}

impl Get {
    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::Array(vec![]);
        frame.push_bytes(b"GET");
        frame
    }

    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        match parse.next_string()?.to_uppercase().as_str() {
            "SESSIONS" => match GetSessions::parse_frames(parse) {
                Ok(get) => Ok(Self::Sessions(get)),
                Err(e) => Err(e),
            },
            _ => Err("Invalid GET command".into()),
        }
    }
}
