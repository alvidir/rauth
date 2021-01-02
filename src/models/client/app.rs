use diesel::NotFound;
use std::error::Error;
use crate::models::client::Controller as ClientController;
use crate::schema::apps;

extern crate diesel;
use crate::diesel::prelude::*;
use crate::postgres::*;

pub trait Controller {
    fn get_description(&self) -> Option<String>;
    fn get_addr(&self) -> &str;
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
pub struct NewApp<'a> {
    pub client_id: i32,
    pub description: Option<&'a str>,
    pub url: &'a str,
}

impl App {
    pub fn create<'a>(client_id: i32, url: &'a str) -> Result<Self, Box<dyn Error>> {
        let new_app = NewApp {
            client_id: client_id,
            url: url,
            description: None,
        };

        let connection = open_stream();
        let result = diesel::insert_into(apps::table)
            .values(&new_app)
            .get_result::<App>(connection)?;

        Ok(result)
    }

    pub fn find_by_id(target: i32) -> Result<Self, Box<dyn Error>>  {
        use crate::schema::apps::dsl::*;

        let connection = open_stream();
        let results = apps.filter(id.eq(target))
            .load::<App>(connection)?;

        if results.len() > 0 {
            Ok(results[0].clone())
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn build(&self, client: Box<dyn ClientController>) -> impl Controller {
        Wrapper::new(self.clone(), client)
    }
}

// A Wrapper stores the relation between an Application and other structs
struct Wrapper{
    data: App,
    owner: Box<dyn ClientController>,
}

impl Wrapper{
    fn new(data: App, client: Box<dyn ClientController>) -> Self {
        Wrapper{
            data: data,
            owner: client,
        }
    }
}

impl Controller for Wrapper {
    fn get_description(&self) -> Option<String> {
        self.data.description.clone()
    }

    fn get_addr(&self) -> &str {
        &self.data.url
    }
}