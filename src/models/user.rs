#![allow(unused)]

use diesel::NotFound;
use std::error::Error;
use crate::models::client;
use crate::models::enums;
use crate::regex::*;
use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::users;
extern crate diesel;

pub trait Ctrl {
    fn get_client_id(&self) -> i32;
    fn get_email(&self) -> &str;
    fn get_name(&self) -> &str;
}

pub fn find_by_id(target: i32) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::users::dsl::*;

    let connection = open_stream();
    let results = users.filter(id.eq(target))
        .load::<User>(connection)?;

    if results.len() > 0 {
        let client = client::find_by_id(results[0].client_id)?;
        let wrapper = results[0].build(client)?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_name<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::users::dsl::*;

    let client = client::find_by_name(target)?;
    if client.get_kind() != enums::Kind::USER {
        let user_str = enums::Kind::USER.to_string();
        let msg = format!("Client {:?} is not of the type {:?}", client.get_name(), user_str);
        return Err(msg.into());
    }

    let connection = open_stream();
    let results = users.filter(client_id.eq(client.get_id()))
        .load::<User>(connection)?;

    if results.len() > 0 {
        let wrapper = results[0].build(client)?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_email<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::users::dsl::*;

    let connection = open_stream();
    let results = users.filter(email.eq(target))
        .load::<User>(connection)?;

    if results.len() > 0 {
        let client = client::find_by_id(results[0].client_id)?;
        let wrapper = results[0].build(client)?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
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

#[derive(Insertable)]
#[table_name="users"]
struct NewUser<'a> {
    pub client_id: i32,
    pub email: &'a str,
}

impl User {
    pub fn new<'a>(name: &'a str, email: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>> {
        match_email(email)?;

        let kind_id = enums::Kind::USER.to_int32();
        let client = client::Client::new(kind_id, name)?;
        let new_user = NewUser {
            client_id: client.get_id(),
            email: email,
        };

        let connection = open_stream();
        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(connection)?;

        let wrapper = result.build(client)?;
        Ok(Box::new(wrapper))
    }

    fn build(&self, client: Box<dyn client::Ctrl>) -> Result<Wrapper, Box<dyn Error>> {
        Ok(Wrapper{
            user: self.clone(),
            client: client,
        })
    }
}

pub struct Wrapper {
    user: User,
    client: Box<dyn client::Ctrl>,
}

impl Ctrl for Wrapper {
    fn get_client_id(&self) -> i32 {
        self.client.get_id()
    }

    fn get_email(&self) -> &str {
        &self.user.email
    }

    fn get_name(&self) -> &str {
        self.client.get_name()
    }
}