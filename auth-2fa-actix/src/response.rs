use serde::Serialize;

use crate::model::User;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: String,

    pub otp_enabled: bool,
    pub otp_verified: bool,
    pub otp_base32: Option<String>,
    pub otp_auth_url: Option<String>,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub status: String,
    pub user: UserData,
}

pub fn user_to_response(user: &User) -> UserData {
    UserData {
        id: user.id.to_owned(),
        name: user.name.to_owned(),
        email: user.email.to_owned(),
        otp_enabled: user.otp_enabled.to_owned(),
        otp_verified: user.otp_verified.to_owned(),
        otp_base32: user.otp_base32.to_owned(),
        otp_auth_url: user.otp_auth_url.to_owned(),
        created_at: user.created_at.unwrap(),
        updated_at: user.updated_at.unwrap(),
    }
}
