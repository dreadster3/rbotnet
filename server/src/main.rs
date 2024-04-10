use std::sync::Arc;

use actix::Actor;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use server::websocket::server::BotServer;
use server::{app, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log_env = env_logger::Env::new().default_filter_or("info");

    env_logger::init_from_env(log_env);

    // Start the bot server
    let bot_server = BotServer::new().start();

    let app_state = AppState::new(bot_server);

    return HttpServer::new(move || {
        return App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .configure(app::routes::register_routes);
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}
