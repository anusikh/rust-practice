use actix_web::{
    post,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use argonautica::{Hasher, Verifier};
use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use rand::Rng;
use serde_json::json;
use sha2::Sha256;
use totp_rs::{Secret, TOTP};
use uuid::Uuid;

use crate::{
    db::Database,
    middleware::auth_middleware::TokenClaims,
    model::{GenerateOTPSchema, User, UserLoginSchema, UserRegisterSchema, VerifyOTPSchema},
    response::{user_to_response, GenericResponse},
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
                        json!({"status": "fail", "message": "could not register", "error": e.to_string()}),
                    );
                }
            }
        }
    }
}

// login user
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
                let token_str: String = claims.sign_with_key(&jwt_secret).unwrap();
                HttpResponse::Ok().json(json!({"status": "pass".to_string(), "jwt_token": token_str, "2FA_enabled": user.otp_enabled}))
            } else {
                let resp = GenericResponse {
                    status: "fail".to_string(),
                    message: "incorrect username or password".to_string(),
                };
                HttpResponse::Unauthorized().json(json!(resp))
            }
        }
        Err(e) => {
            let resp = GenericResponse {
                status: "fail".to_string(),
                message: format!("incorrect username or password {}", e.to_string()),
            };
            return HttpResponse::Conflict().json(resp);
        }
    }
}

#[post("/auth/otp/generate")]
async fn generate_otp_handler(
    data: web::Data<Database>,
    req_user: Option<ReqData<TokenClaims>>,
    body: web::Json<GenerateOTPSchema>,
) -> impl Responder {
    match req_user {
        Some(u) => {
            let user_exists = data.if_user_exists_userid(&u.id);
            match user_exists {
                true => {
                    let mut rng = rand::thread_rng();
                    let data_byte: [u8; 21] = rng.gen();

                    let base32_string =
                        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data_byte);
                    let totp = TOTP::new(
                        totp_rs::Algorithm::SHA1,
                        6,
                        1,
                        30,
                        Secret::Encoded(base32_string).to_bytes().unwrap(),
                    )
                    .unwrap();

                    let otp_base32 = totp.get_secret_base32();
                    let email = body.email.to_owned();
                    let issuer = "anusikh";
                    let otp_auth_url = format!(
                        "otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}"
                    );

                    let res = data.update_totp_for_user(&u.id, &otp_base32, &otp_auth_url, false);
                    match res {
                        Ok(_) => HttpResponse::Ok().json(json!(GenericResponse {
                            status: "pass".to_string(),
                            message: format!(
                                "base32: {}, otp_auth_url: {}",
                                otp_base32, otp_auth_url
                            )
                        })),
                        Err(e) => HttpResponse::NotFound().json(json!(GenericResponse {
                            status: "fail".to_string(),
                            message: format!("something went wrong: {}", e.to_string())
                        })),
                    }
                }
                false => {
                    let resp = GenericResponse {
                        status: "fail".to_string(),
                        message: format!("cannot find user {}", u.id),
                    };
                    HttpResponse::NotFound().json(resp)
                }
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[post("/auth/otp/verify")]
async fn verify_otp_handler(
    data: web::Data<Database>,
    req_user: Option<ReqData<TokenClaims>>,
    body: web::Json<VerifyOTPSchema>,
) -> impl Responder {
    match req_user {
        Some(u) => {
            let user = data.get_user_by_userid(&u.id);
            match user {
                Ok(us) => {
                    let totp = TOTP::new(
                        totp_rs::Algorithm::SHA1,
                        6,
                        1,
                        30,
                        Secret::Encoded(us.otp_base32.unwrap()).to_bytes().unwrap(),
                    )
                    .unwrap();

                    let is_valid = totp.check_current(&body.token).unwrap();
                    if !is_valid {
                        let json_error = GenericResponse {
                            status: "fail".to_string(),
                            message: "Token is invalid or user doesn't exist".to_string(),
                        };
                        return HttpResponse::Forbidden().json(json_error);
                    }

                    let res = data.update_totp_for_user(&u.id, "", "", true);
                    match res {
                        Ok(usr) => HttpResponse::Ok().json(json!(
                            {"status":"pass","otp_verified": true, "user": user_to_response(&usr)}
                        )),
                        Err(e) => HttpResponse::NotFound().json(GenericResponse {
                            status: "fail".to_string(),
                            message: format!("something went wrong {}", e.to_string()),
                        }),
                    }
                }
                Err(e) => HttpResponse::NotFound().json(GenericResponse {
                    status: "fail".to_string(),
                    message: format!("something went wrong {}", e.to_string()),
                }),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[post("/auth/otp/validate")]
async fn validate_otp_handler(
    data: web::Data<Database>,
    req_user: Option<ReqData<TokenClaims>>,
    body: web::Json<VerifyOTPSchema>,
) -> impl Responder {
    match req_user {
        Some(u) => {
            let user = data.get_user_by_userid(&u.id);
            match user {
                Ok(us) => {
                    if !us.otp_enabled {
                        let json_error = GenericResponse {
                            status: "fail".to_string(),
                            message: "2FA not enabled".to_string(),
                        };

                        return HttpResponse::Forbidden().json(json_error);
                    }

                    let totp = TOTP::new(
                        totp_rs::Algorithm::SHA1,
                        6,
                        1,
                        30,
                        Secret::Encoded(us.otp_base32.unwrap()).to_bytes().unwrap(),
                    )
                    .unwrap();

                    let is_valid = totp.check_current(&body.token).unwrap();
                    if !is_valid {
                        let json_error = GenericResponse {
                            status: "fail".to_string(),
                            message: "Token is invalid or user doesn't exist".to_string(),
                        };
                        return HttpResponse::Forbidden().json(json_error);
                    }

                    HttpResponse::Ok().json(GenericResponse {
                        status: "pass".to_string(),
                        message: "verified".to_string(),
                    })
                }
                Err(e) => HttpResponse::NotFound().json(GenericResponse {
                    status: "fail".to_string(),
                    message: format!("something went wrong {}", e.to_string()),
                }),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
