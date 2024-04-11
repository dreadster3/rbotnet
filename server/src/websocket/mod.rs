pub mod messages;
pub mod server;
pub mod session;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;
