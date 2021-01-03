use diesel::NotFound;
use std::error::Error;
use crate::schema::apps;
use crate::models::client::Extension;
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
    pub fn create<'a>(client_id: i32, url: &'a str) -> Result<impl Extension, Box<dyn Error>> {
        match_url(url)?;
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

    pub fn find_by_id(target: i32) -> Result<impl Extension, Box<dyn Error>>  {
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

    pub fn find_by_client_id(target: i32) -> Result<impl Extension, Box<dyn Error>>  {
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

    pub fn build(&self/*, client: Box<dyn ClientController>*/) -> impl Extension {
        Wrapper::new(self.clone()/*, client*/)
    }
}

impl Extension for App {
    fn get_addr(&self) -> String {
        self.url.clone()
    }

    fn get_client_id(&self) -> i32 {
        self.client_id
    }
}

// A Wrapper stores the relation between an Application and other structs
struct Wrapper{
    data: App,
    /*owner: Box<dyn ClientController>,*/
}

impl Wrapper{
    fn new(data: App/*, client: Box<dyn ClientController>*/) -> Self {
        Wrapper{
            data: data,
            /*owner: client,*/
        }
    }
}

impl Extension for Wrapper {
    fn get_addr(&self) -> String {
        self.data.url.clone()
    }

    fn get_client_id(&self) -> i32 {
        self.data.client_id
    }
}