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
    pub fn new(client: Box<dyn ClientController>, email: String) -> impl Controller {
        let user = User {
            id: 0,
            client_id: client.get_id(),
            email: email,
        };

        Wrapper::build(user, client)
    }

    pub fn find_user_by_id(target: i32, client: Box<dyn ClientController>) -> Result<Option<Box<dyn Controller>>, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;

        let connection = open_stream();
        let results = users.filter(id.eq(target))
            .load::<User>(connection)?;

        if results.len() > 0 {
            let user = results[0].clone();
            let wrapp = Wrapper::build(user, client);
            Ok(Some(Box::new(wrapp)))
        } else {
            Ok(None)
        }
    }
}

// A Wrapper makes the relation between a Client and other structs
struct Wrapper{
    data: User,
    owner: Box<dyn ClientController>,
}

impl Wrapper{
    fn build(data: User, client: Box<dyn ClientController>) -> Self {
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