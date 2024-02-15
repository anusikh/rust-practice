use actix_web::{post, web, HttpResponse, Responder};
use argonautica::{Hasher, Verifier};
use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;

use crate::{
    db::Database,
    middleware::auth_middleware::TokenClaims,
    model::{User, UserLoginSchema, UserRegisterSchema},
    response::GenericResponse,
};

// register user
#[post("/auth/register")]
async fn register_user_handler(
    data: web::Data<Database>,
    body: web::Json<UserRegisterSchema>,
) -> impl Responder {
    let exiting_user = data.if_user_exists(&body.email);
    match exiting_user {
        true => {
            let resp = GenericResponse {
                status: "fail".to_string(),
                message: format!("user with email {} already exists", &body.email),
            };
            return HttpResponse::Conflict().json(resp);
        }
        false => {
            let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set");
            let mut hasher = Hasher::default();
            let hash = hasher
                .with_password(body.password.to_owned())
                .with_secret_key(hash_secret)
                .hash()
                .unwrap();

            let uuid = Uuid::new_v4();
            let date_time = Utc::now().naive_utc();

            let user = User {
                id: uuid.to_string(),
                email: body.email.to_owned(),
                name: body.name.to_owned(),
                password: hash,
                otp_enabled: false,
                otp_verified: false,
                otp_base32: None,
                otp_auth_url: None,
                created_at: Some(date_time),
                updated_at: Some(date_time),
            };

            let final_result: Result<usize, diesel::result::Error> = data.add_user(user);
            match final_result {
                Ok(_) => HttpResponse::Ok()
                    .json(json!({"status": "success", "message": "registration successful"})),
                Err(e) => {
                    return HttpResponse::Conflict().json(
                        json!({"status": "failed", "message": "could not register", "error": e.to_string()}),
                    );
                }
            }
        }
    }
}

#[post("/auth/login")]
async fn login_user_handler(
    data: web::Data<Database>,
    body: web::Json<UserLoginSchema>,
) -> impl Responder {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .as_bytes(),
    )
    .unwrap();

    let email = body.email.to_owned();
    let password = body.password.to_owned();

    let user_from_db = data.get_user_by_email(&email);
    match user_from_db {
        Ok(user) => {
            let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set");
            let mut verifier = Verifier::default();

            let is_valid = tokio::spawn(async move {
                let res: bool = verifier
                    .with_hash(user.password)
                    .with_password(password)
                    .with_secret_key(hash_secret)
                    .verify()
                    .unwrap();
                res
            })
            .await
            .unwrap();

            if is_valid {
                let claims: TokenClaims = TokenClaims { id: user.id };
                let token_str = claims.sign_with_key(&jwt_secret).unwrap();
                let resp = GenericResponse {
                    status: "pass".to_string(),
                    message: token_str,
                };
                HttpResponse::Ok().json(resp)
            } else {
                let resp = GenericResponse {
                    status: "failed".to_string(),
                    message: "incorrect username or password".to_string(),
                };
                HttpResponse::Unauthorized().json(json!(resp))
            }
        }
        Err(e) => {
            let resp = GenericResponse {
                status: "failed".to_string(),
                message: format!("incorrect username or password {}", e.to_string()),
            };
            return HttpResponse::Conflict().json(resp);
        }
    }
}
