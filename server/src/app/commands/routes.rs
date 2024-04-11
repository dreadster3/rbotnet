use crate::websocket::messages::BroadcastCommand;

use crate::AppState;
use actix_web::{post, web, HttpResponse};

use protocol::commands::command::Command;

type Result = std::result::Result<HttpResponse, actix_web::Error>;

#[post("heartbeat")]
pub async fn heartbeat(state: web::Data<AppState>) -> Result {
    let server = state.server();
    let command = Command::Heartbeat;
    let sessions = match server.send(BroadcastCommand(command)).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().json(sessions));
}

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("commands").service(heartbeat);

    cfg.service(scope);
}
