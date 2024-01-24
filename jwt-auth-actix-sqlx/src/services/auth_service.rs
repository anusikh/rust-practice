use actix_web::{
    get, post,
    web::{Data, Json, ReqData},
    HttpResponse, Responder,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use argonautica::{Hasher, Verifier};

use chrono::NaiveDateTime;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{middlewares::auth_middleware::TokenClaims, AppState};

#[derive(Deserialize)]
struct CreateUserBody {
    username: String,
    password: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct UserNoPassword {
    id: u64,
    username: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct AuthUser {
    id: u64,
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateArticleBody {
    title: String,
    content: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Article {
    id: u64,
    title: String,
    content: String,
    lished_by: i32,
    lished_on: Option<NaiveDateTime>,
}

#[post("/user")]
async fn create_user(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
    let user: CreateUserBody = body.into_inner();

    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set");
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(user.password)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    match sqlx::query(
        "
    INSERT INTO users (username, password) VALUES (?, ?);
    ",
    )
    .bind(user.username)
    .bind(hash)
    .execute(&state.db)
    .await
    {
        Ok(_) => HttpResponse::Ok().json(format!("{:?}", "done")),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

#[get("/auth")]
async fn basic_auth(state: Data<AppState>, credentials: BasicAuth) -> impl Responder {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set!")
            .as_bytes(),
    )
    .unwrap();
    let username = credentials.user_id();
    let password = credentials.password();

    match password {
        None => HttpResponse::Unauthorized().json("Must provide username and password"),
        Some(pass) => {
            match sqlx::query_as::<_, AuthUser>(
                "SELECT id, username, password FROM users WHERE username = ?",
            )
            .bind(username.to_string())
            .fetch_one(&state.db)
            .await
            {
                Ok(user) => {
                    let hash_secret =
                        std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
                    let mut verifier = Verifier::default();
                    let is_valid = verifier
                        .with_hash(user.password)
                        .with_password(pass)
                        .with_secret_key(hash_secret)
                        .verify()
                        .unwrap();

                    if is_valid {
                        let claims = TokenClaims { id: user.id };
                        let token_str = claims.sign_with_key(&jwt_secret).unwrap();
                        HttpResponse::Ok().json(token_str)
                    } else {
                        HttpResponse::Unauthorized().json("Incorrect username or password")
                    }
                }
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
    }
}

#[post("/article")]
async fn create_article(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<CreateArticleBody>,
) -> impl Responder {
    match req_user {
        Some(user) => {
            let article: CreateArticleBody = body.into_inner();

            match sqlx::query(
                "INSERT INTO articles (title, content, published_by)
                VALUES (?, ?, ?);
                ",
            )
            .bind(article.title)
            .bind(article.content)
            .bind(user.id)
            .execute(&state.db)
            .await
            {
                Ok(_) => HttpResponse::Ok().json(format!("{:?}", "done")),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
