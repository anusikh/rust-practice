// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        otp_enabled -> Bool,
        otp_verified -> Bool,
        #[max_length = 255]
        otp_base32 -> Nullable<Varchar>,
        #[max_length = 255]
        otp_auth_url -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
