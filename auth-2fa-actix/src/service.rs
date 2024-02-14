use actix_web::{post, web, HttpResponse, Responder};
use argonautica::Hasher;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::Database,
    model::{User, UserRegisterSchema},
    response::GenericResponse,
};

// register user
#[post("/auth/register")]
async fn register_user_handler(
    data: web::Data<Database>,
    body: web::Json<UserRegisterSchema>,
) -> impl Responder {
    let exiting_user = data.get_existing_user(&body.email);
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
