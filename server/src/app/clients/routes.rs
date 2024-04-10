use crate::websocket::{messages::ListSessions, session::BotSession};
use crate::AppState;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;

type Result = std::result::Result<HttpResponse, actix_web::Error>;

#[get("")]
pub async fn index(state: web::Data<AppState>) -> Result {
    let server = state.server();
    if let Ok(sessions) = server.send(ListSessions).await {
        return Ok(HttpResponse::Ok().json(sessions));
    }

    return Ok(HttpResponse::InternalServerError().finish());
}

#[get("ws")]
pub async fn websocket(
    req: HttpRequest,
    state: web::Data<AppState>,
    stream: web::Payload,
) -> Result {
    let peer_address = req.peer_addr().unwrap();
    let session = BotSession::new(state.server(), peer_address);

    ws::start(session, &req, stream)
}

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("clients")
        .service(index)
        .service(websocket);

    cfg.service(scope);
}
