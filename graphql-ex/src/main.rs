use actix_web::{
    guard,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use config::mongo::DbMongo;
use handler::graphql_handler::{GraphqlSchema, Mutation, Query};

mod config;
mod handler;
mod schemas;

//graphql entry
async fn index(schema: Data<GraphqlSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = DbMongo::init().await;
    let schema_data = Schema::build(Query, Mutation, EmptySubscription)
        .data(db) // in handler, we cam get this data in context
        .finish();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema_data.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index)) // post requests
            .service(
                web::resource("/")
                    .guard(guard::Get()) // playground on get request
                    .to(graphql_playground),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
