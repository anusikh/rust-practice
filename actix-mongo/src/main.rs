mod api;
mod models;
mod repository;

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use api::user_api::{create_user, delete_user, get_all_users_paginated, get_user, update_user};
use repository::mongodb_repo::MongoRepo;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello from rust and mongodb")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(hello)
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_all_users_paginated)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
