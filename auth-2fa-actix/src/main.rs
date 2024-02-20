pub mod db;
pub mod middleware;
pub mod model;
pub mod response;
pub mod schema;
pub mod service;

use actix_cors::Cors;
use actix_web::{
    get,
    http::header,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;

use crate::{
    db::Database,
    middleware::auth_middleware::validator,
    service::{
        disable_otp_handler, generate_otp_handler, login_user_handler, register_user_handler,
        validate_otp_handler, verify_otp_handler,
    },
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
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();

        let bearer_middleware = HttpAuthentication::bearer(validator);

        App::new()
            .app_data(Data::new(Database::new()))
            .service(health_check_handler)
            .service(login_user_handler)
            .service(register_user_handler)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(generate_otp_handler)
                    .service(verify_otp_handler)
                    .service(validate_otp_handler)
                    .service(disable_otp_handler),
            )
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
