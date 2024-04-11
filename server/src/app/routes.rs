use super::{clients, commands};

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let scope = actix_web::web::scope("api")
        .configure(clients::routes::register_routes)
        .configure(commands::routes::register_routes);

    cfg.service(scope);
}
