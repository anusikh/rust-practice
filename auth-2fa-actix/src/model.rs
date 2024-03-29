use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::users )]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password: String,

    pub otp_enabled: bool,
    pub otp_verified: bool,
    pub otp_base32: Option<String>,
    pub otp_auth_url: Option<String>,

    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct UserRegisterSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLoginSchema {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateOTPSchema {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOTPSchema {
    pub token: String,
}
