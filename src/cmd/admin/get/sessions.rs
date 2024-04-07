use log::info;

use crate::{
    cmd::{
        frame::{Frame, Serialize},
        parse::Parse,
    },
    server::state::{Connection, State},
};

pub struct GetSessions {}

impl GetSessions {
    pub fn new() -> Self {
        Self {}
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::Array(vec![]);
        frame.push_bytes(b"GET SESSIONS");
        frame
    }

    pub fn parse_frames(_: &mut Parse) -> crate::Result<Self> {
        Ok(Self::new())
    }

    pub async fn execute(
        &self,
        connection: &Connection,
        client_connections: State,
    ) -> crate::Result<()> {
        info!(target: "admin_session_events", "Executing GetSessions");
        let connections = client_connections.clone();
        let connections = connections.lock().await;
        let mut frame = Frame::new();

        for connection in connections.values() {
            info!(target: "admin_session_events", session_id=connection.get_id(); "Session: {:?}", connection.get_address());
            frame.push_string(format!("Session: {}", connection.get_address()));
        }

        connection
            .get_sender()
            .send(frame.serialize().unwrap())
            .unwrap();

        Ok(())
    }
}

impl Into<Frame> for GetSessions {
    fn into(self) -> Frame {
        self.into_frame()
    }
}
