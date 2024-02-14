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

    pub fn get_existing_user(&self, entered_email: &str) -> bool {
        let res = select(exists(users.filter(email.eq(entered_email))))
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
}
