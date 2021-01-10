use diesel::NotFound;
use std::error::Error;
use crate::schema::apps;
use crate::models::client::{Client, Extension, Controller as ClientController};
use crate::models::kind::{KIND_APP, Kind, Controller as KindController};
use crate::regex::*;

extern crate diesel;
use crate::diesel::prelude::*;
use crate::postgres::*;

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
pub struct NewApp<'a> {
    pub client_id: i32,
    pub description: Option<&'a str>,
    pub url: &'a str,
}

impl App {
    pub fn create<'a>(name: &'a str, url: &'a str, pwd: &'a str) -> Result<Box<dyn ClientController>, Box<dyn Error>> {
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

    pub fn find_by_id(target: i32) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
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

    pub fn find_by_name<'a>(target: &'a str) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
        let kind = Kind::find_by_name(KIND_APP)?;
        Client::find_kind_by_name(target, Box::new(kind))
    }


    pub fn find_by_url<'a>(target: &'a str) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
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

    pub fn find_by_client_id(target: i32) -> Result<Box<dyn ClientController>, Box<dyn Error>>  {
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

    fn build(&self) -> Result<Box<dyn ClientController>, Box<dyn Error>> {
        Client::from_extension(Box::new(self.clone()))
    }
}

impl Extension for App {
    fn get_addr(&self) -> String {
        self.url.clone()
    }

    fn get_client_id(&self) -> i32 {
        self.client_id
    }

    fn get_kind(&self) -> &str {
        KIND_APP
    }
}

pub fn find_as_extension(target: i32) -> Result<impl Extension, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let connection = open_stream();
    let results = apps.filter(client_id.eq(target))
        .load::<App>(connection)?;

    if results.len() > 0 {
        Ok(results[0].clone())
    } else {
        Err(Box::new(NotFound))
    }
}