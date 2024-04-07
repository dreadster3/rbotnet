use crate::server::state::{Connection, State};

use super::{admin::get::Get, frame::Frame, parse::Parse};

pub enum Command {
    Get(Get),
}

impl Command {
    pub fn into_frame(self) -> Frame {
        match self {
            Command::Get(get) => get.into_frame(),
        }
    }

    pub fn from_frame(frame: Frame) -> crate::Result<Self> {
        let mut parse = Parse::new(frame)?;

        let command = parse.next_string()?.to_uppercase();

        match command.as_str() {
            "GET" => match Get::parse_frames(&mut parse) {
                Ok(get) => Ok(Self::Get(get)),
                Err(e) => Err(e),
            },
            _ => Err("Invalid command".into()),
        }
    }

    pub async fn execute(
        &self,
        connection: &Connection,
        client_connections: State,
    ) -> crate::Result<()> {
        match self {
            Command::Get(get) => match get {
                Get::Sessions(get) => get.execute(connection, client_connections.clone()).await,
            },
        }
    }
}

impl Into<Frame> for Command {
    fn into(self) -> Frame {
        self.into_frame()
    }
}
