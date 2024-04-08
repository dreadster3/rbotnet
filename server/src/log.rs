use log::kv::{Key, Value};
use structured_logger::Writer;

pub struct MultiWriter {
    writers: Vec<Box<dyn Writer>>,
}

impl MultiWriter {
    pub fn new(writers: Vec<Box<dyn Writer>>) -> Self {
        Self { writers }
    }
}

impl Writer for MultiWriter {
    fn write_log(
        &self,
        value: &std::collections::BTreeMap<Key, Value>,
    ) -> Result<(), std::io::Error> {
        for writer in &self.writers {
            writer.write_log(value)?;
        }

        Ok(())
    }
}

pub mod log_targets {
    pub const SESSION_EVENTS_TARGET: &str = "session_events";
    pub const SERVER_EVENTS_TARGET: &str = "server_events";
}
