use std::error::Error;
use crate::models::client::Controller as ClientController;

extern crate diesel;
use crate::diesel::prelude::*;
use crate::postgres::*;

use crate::schema::users;

pub trait Controller {
    fn get_addr(&self) -> &str;
}

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub client_id: i32,
    pub email: String,
}

impl User {
    pub fn find_by_id(target: i32) -> Result<Option<Self>, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;

        let connection = open_stream();
        let results = users.filter(id.eq(target))
            .load::<User>(connection)?;

        if results.len() > 0 {
            Ok(Some(results[0].clone()))
        } else {
            Ok(None)
        }
    }

    pub fn build(&self, client: Box<dyn ClientController>) -> impl Controller {
        Wrapper::new(self.clone(), client)
    }
}

// A Wrapper stores the relation between a Client and other structs
struct Wrapper{
    data: User,
    owner: Box<dyn ClientController>,
}

impl Wrapper{
    fn new(data: User, client: Box<dyn ClientController>) -> Self {
        Wrapper{
            data: data,
            owner: client,
        }
    }
}

impl Controller for Wrapper {
    fn get_addr(&self) -> &str {
        &self.data.email
    }

}