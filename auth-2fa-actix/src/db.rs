use diesel::dsl::exists;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error;
use diesel::{prelude::*, select};
use dotenv::dotenv;

use crate::model::User;
use crate::schema::users::dsl::*;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url: String =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is not available, add env");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool: DbPool = r2d2::Pool::builder()
            .build(manager)
            .expect("failed to create pool");
        Database { pool }
    }

    pub fn if_user_exists(&self, entered_email: &str) -> bool {
        let res = select(exists(users.filter(email.eq(entered_email))))
            .get_result::<bool>(&mut self.pool.get().unwrap())
            .expect("something went wrong");
        res
    }

    pub fn if_user_exists_userid(&self, user_id: &str) -> bool {
        let res = select(exists(users.filter(id.eq(user_id))))
            .get_result::<bool>(&mut self.pool.get().unwrap())
            .expect("something went wrong");
        res
    }

    pub fn add_user(&self, user: User) -> Result<usize, Error> {
        let res = diesel::insert_into(users)
            .values(&user)
            .returning(User::as_returning())
            .execute(&mut self.pool.get().unwrap());
        res
    }

    pub fn get_user_by_email(&self, user_email: &str) -> Result<User, Error> {
        let res = users
            .filter(email.eq(user_email))
            .get_result::<User>(&mut self.pool.get().unwrap());
        res
    }

    pub fn get_user_by_userid(&self, user_id: &str) -> Result<User, Error> {
        let res = users
            .filter(id.eq(user_id))
            .get_result::<User>(&mut self.pool.get().unwrap());
        res
    }

    pub fn update_totp_for_user(
        &self,
        user_id: &str,
        gen_otp_base32: &str,
        gen_otp_auth_url: &str,
        verified: bool,
    ) -> Result<User, Error> {
        match verified {
            true => {
                let res = diesel::update(users.find(&user_id))
                    .set((otp_enabled.eq(verified), otp_verified.eq(verified)))
                    .get_result::<User>(&mut self.pool.get().unwrap());
                res
            }
            false => {
                let res: Result<User, Error> = diesel::update(users.find(&user_id))
                    .set((
                        otp_base32.eq(&gen_otp_base32),
                        otp_auth_url.eq(&gen_otp_auth_url),
                        otp_base32.eq(&gen_otp_base32),
                        otp_auth_url.eq(&gen_otp_auth_url),
                    ))
                    .get_result::<User>(&mut self.pool.get().unwrap());
                res
            }
        }
    }
}
