pub mod db;
pub mod middleware;
pub mod model;
pub mod response;
pub mod schema;
pub mod service;

use actix_web::{
    get,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;

use crate::{
    db::Database,
    middleware::auth_middleware::validator,
    service::{generate_otp_handler, login_user_handler, register_user_handler},
};

#[get("/api/health")]
async fn health_check_handler() -> impl Responder {
    const MESSAGE: &str = "Hello 2FA";
    HttpResponse::Ok().json(json!({"status":"success", "message": MESSAGE}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    env_logger::init();

    println!("Server started successfully");

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);

        App::new()
            .app_data(Data::new(Database::new()))
            .service(health_check_handler)
            .service(login_user_handler)
            .service(register_user_handler)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(generate_otp_handler),
            )
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
