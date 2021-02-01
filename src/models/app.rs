use diesel::NotFound;
use std::error::Error;
use crate::schema::apps;
use crate::models::client::Client;
use crate::models::kind;
use crate::regex::*;
use crate::diesel::prelude::*;
use crate::postgres::*;
extern crate diesel;

pub trait Ctrl {
    fn get_url(&self) -> &str;
}

pub fn find_by_id(target: i32) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let connection = open_stream();
    let results = apps.filter(id.eq(target))
        .load::<App>(connection)?;

    if results.len() > 0 {
        results[0].build()
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_name<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    let kind = Kind::find_by_name(KIND_APP)?;
    Client::find_kind_by_name(target, Box::new(kind))
}


pub fn find_by_url<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let connection = open_stream();
    let results = apps.filter(url.eq(target))
        .load::<App>(connection)?;

    if results.len() > 0 {
        results[0].build()
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_client_id(target: i32) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let connection = open_stream();
    let results = apps.filter(client_id.eq(target))
        .load::<App>(connection)?;

    if results.len() > 0 {
        let app = results[0].clone();
        Client::from_extension(Box::new(app))
    } else {
        Err(Box::new(NotFound))
    }
}

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct App {
    pub id: i32,
    pub client_id: i32,
    pub description: Option<String>,
    pub url: String,
}

#[derive(Insertable)]
#[table_name="apps"]
struct NewApp<'a> {
    pub client_id: i32,
    pub description: Option<&'a str>,
    pub url: &'a str,
}

impl App {
    pub fn new<'a>(name: &'a str, url: &'a str, pwd: &'a str) -> Result<Self, Box<dyn Error>> {
        match_url(url)?;

        let kind = Kind::find_by_name(KIND_APP)?;
        let mut client = Client::create(kind.get_id(), name, pwd)?;
        let new_app = NewApp {
            client_id: client.get_id(),
            url: url,
            description: None,
        };

        let connection = open_stream();
        let result = diesel::insert_into(apps::table)
            .values(&new_app)
            .get_result::<App>(connection)?;

        client.set_extension(Box::new(result))?;
        Ok(client)
    }
}