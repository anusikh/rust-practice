mod middlewares;
mod services;

use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use middlewares::auth_middleware::validator;
use services::auth_service::{basic_auth, create_article, create_user};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub struct AppState {
    db: Pool<MySql>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not missing");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("couldn't build connection pool");

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin();
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .wrap(cors)
            .service(basic_auth)
            .service(create_user)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(create_article),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
