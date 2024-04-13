use crate::websocket::messages::Disconnect;
use crate::websocket::{messages::ListSessions, session::BotSession};
use crate::AppState;
use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;

type Result = std::result::Result<HttpResponse, actix_web::Error>;

#[get("")]
pub async fn index(state: web::Data<AppState>) -> Result {
    let server = state.server();
    let sessions = match server.send(ListSessions).await {
        Ok(sessions) => sessions,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().json(sessions));
}

#[delete("{id}")]
pub async fn close_session(state: web::Data<AppState>, id: web::Path<String>) -> Result {
    let client_id = id.into_inner();
    let server = state.server();
    let command = Disconnect(client_id);

    match server.send(command).await {
        Ok(_) => (),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    return Ok(HttpResponse::Ok().body("Disconnect signal sent"));
}

#[get("ws")]
pub async fn websocket(
    req: HttpRequest,
    state: web::Data<AppState>,
    stream: web::Payload,
) -> Result {
    let peer_address = req.peer_addr().unwrap();
    let semaphore = state.semaphore();
    let permit = match semaphore.try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return Ok(HttpResponse::ServiceUnavailable().finish());
        }
    };

    let session = BotSession::new(state.server(), peer_address, permit);

    ws::start(session, &req, stream)
}

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("sessions")
        .service(index)
        .service(close_session)
        .service(websocket);

    cfg.service(scope);
}
