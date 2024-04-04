use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

type Sender = UnboundedSender<String>;

pub type State = Arc<Mutex<HashMap<String, Sender>>>;

pub fn new() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn add_session(state: State, id: &str, transmitter: Sender) {
    let mut state = state.lock().await;
    state.insert(id.to_string(), transmitter);
}
