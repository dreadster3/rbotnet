use crate::websocket::{messages::ListSessions, session::BotSession};
use crate::AppState;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
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
    let scope = actix_web::web::scope("clients")
        .service(index)
        .service(websocket);

    cfg.service(scope);
}
