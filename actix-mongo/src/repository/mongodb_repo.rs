use std::env;

use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Error,
    options::{ClientOptions, FindOptions},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Collection,
};

use crate::models::user_model::User;

pub struct MongoRepo {
    col: Collection<User>,
}

impl MongoRepo {
    // initialize db
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("error loading env variable"),
        };
        let client_options_res: Result<ClientOptions, Error> = ClientOptions::parse(uri).await;
        match client_options_res {
            Ok(x) => {
                let client: Client = Client::with_options(x).unwrap();
                let db = client.database("rustDB");
                let col: Collection<User> = db.collection("User");
                MongoRepo { col }
            }
            Err(_) => todo!(),
        }
    }

    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            name: new_user.name,
            location: new_user.location,
            title: new_user.title,
        };

        let user: Result<InsertOneResult, Error> = self.col.insert_one(new_doc, None).await;
        match user {
            Ok(x) => Ok(x),
            Err(e) => Err(e),
        }
    }

    pub async fn get_user(&self, id: &String) -> Result<User, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .col
            .find_one(filter, None)
            .await
            .ok()
            .expect("error getting user's detail");
        Ok(user_detail.unwrap())
    }

    pub async fn update_user(&self, id: &String, new_user: User) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let new_doc = doc! {
            "$set":{
                "id": new_user.id,
                "name": new_user.name,
                "location": new_user.location,
                "title": new_user.title
            }
        };
        let updated_doc = self
            .col
            .update_one(filter, new_doc, None)
            .await
            .ok()
            .expect("error updating user");
        Ok(updated_doc)
    }

    pub async fn delete_user(&self, id: &String) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let del_res = self
            .col
            .delete_one(filter, None)
            .await
            .ok()
            .expect("couldn't delete item");
        Ok(del_res)
    }

    // get all data with pagination
    pub async fn get_all_users(&self, page: i64, page_size: i64) -> Result<Vec<User>, Error> {
        let find_options = FindOptions::builder()
            .limit(page_size)
            .skip(u64::try_from((page - 1) * page_size).unwrap())
            .build();
        let mut users: Vec<User> = Vec::new();
        let mut cursors = self
            .col
            .find(None, find_options)
            .await
            .ok()
            .expect("error getting all the users");
        while let Some(user) = cursors
            .try_next()
            .await
            .ok()
            .expect("error mapping through")
        {
            users.push(user)
        }
        Ok(users)
    }
}
