use diesel::NotFound;
use std::error::Error;
use crate::models::client::{Client, Extension, Controller as ClientController};
use crate::models::kind::{KIND_USER, Kind, Controller as KindController};
use crate::regex::*;

extern crate diesel;
use crate::diesel::prelude::*;
use crate::postgres::*;

use crate::schema::users;

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub client_id: i32,
    pub email: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub client_id: i32,
    pub email: &'a str,
}

impl User {
    pub fn create<'a>(name: &'a str, email: &'a str, pwd: &'a str) -> Result<Box<dyn ClientController>, Box<dyn Error>> {
        match_email(email)?;
        let kind = Kind::find_by_name(KIND_USER)?;
        let mut client = Client::create(kind.get_id(), name, pwd)?;
        let new_user = NewUser {
            client_id: client.get_id(),
            email: email,
        };

        let connection = open_stream();
        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(connection)?;

        client.set_extension(Box::new(result))?;
        Ok(client)
    }

    pub fn find_by_id(target: i32) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;

        let connection = open_stream();
        let results = users.filter(id.eq(target))
            .load::<User>(connection)?;

        if results.len() > 0 {
            results[0].build()
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn find_by_email<'a>(target: &'a str) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;

        let connection = open_stream();
        let results = users.filter(email.eq(target))
            .load::<User>(connection)?;

        if results.len() > 0 {
            results[0].build()
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn find_by_client_id(target: i32) -> Result<impl Extension, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;

        let connection = open_stream();
        let results = users.filter(client_id.eq(target))
            .load::<User>(connection)?;

        if results.len() > 0 {
            Ok(results[0].clone())
        } else {
            Err(Box::new(NotFound))
        }
    }

    fn build(&self) -> Result<Box<dyn ClientController>, Box<dyn Error>> {
        Client::from_extension(Box::new(self.clone()))
    }
}

impl Extension for User {
    fn get_addr(&self) -> String {
        self.email.clone()
    }

    fn get_client_id(&self) -> i32 {
        self.client_id
    }
}