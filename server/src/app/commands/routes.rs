use crate::websocket::messages::{BroadcastCommand, SendCommand};

use crate::AppState;
use actix_web::{post, web, FromRequest, HttpResponse};

use protocol::commands::heartbeat::Heartbeat;

type Result = std::result::Result<HttpResponse, actix_web::Error>;

#[post("heartbeat")]
pub async fn heartbeat(state: web::Data<AppState>) -> Result {
    let server = state.server();
    let command = Heartbeat::new();
    match server.send(BroadcastCommand(command)).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().body("Heartbeat sent"));
}

#[post("heartbeat/{id}")]
pub async fn heartbeat_by_id(state: web::Data<AppState>, id: web::Path<String>) -> Result {
    let client_id = id.into_inner();
    let server = state.server();
    let command = Heartbeat::new();
    match server.send(SendCommand(client_id, command)).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().body("Heartbeat sent"));
}

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("commands")
        .service(heartbeat)
        .service(heartbeat_by_id);

    cfg.service(scope);
}
