use crate::models::client::{Client, Controller as ClientController};

extern crate diesel;
use crate::diesel::prelude::*;

use crate::schema::users;
use crate::schema::users::dsl::users as all_users;

pub trait Controller {
    fn get_addr(&self) -> &str;
}

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub client_id: i32,
    pub email: String,
}

impl User {
    pub fn find(target: i32, conn: &PgConnection) -> Vec<User> {
        all_users.filter(users::id.eq(target))
            .load::<User>(conn)
            .expect("Error while loading user")
    }
}

impl Controller for User {
    fn get_addr(&self) -> &str {
        &self.email
    }

}