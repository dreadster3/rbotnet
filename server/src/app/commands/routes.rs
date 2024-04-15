use crate::websocket::messages::{BroadcastCommand, SendCommand};

use crate::AppState;
use actix_web::{post, web, FromRequest, HttpResponse};

use protocol::commands::heartbeat::Heartbeat;
use protocol::commands::request::Request;

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

#[post("request")]
pub async fn request(state: web::Data<AppState>) -> Result {
    let server = state.server();
    let command = Request::new("GET", "http://localhost:8080/api/sessions", None);
    match server.send(BroadcastCommand(command)).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().body("Request command sent"));
}

#[post("request/{id}")]
pub async fn request_by_id(state: web::Data<AppState>, id: web::Path<String>) -> Result {
    let client_id = id.into_inner();
    let server = state.server();
    let command = Request::new("GET", "http://localhost:8080/api/sessions", None);
    match server.send(SendCommand(client_id, command)).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().body("Request command sent"));
}

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("commands")
        .service(heartbeat)
        .service(heartbeat_by_id)
        .service(request)
        .service(request_by_id);

    cfg.service(scope);
}
