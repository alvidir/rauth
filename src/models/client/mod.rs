mod app;
mod user;

use diesel::NotFound;
use std::error::Error;
use std::time::SystemTime;
use crate::diesel::prelude::*;
use crate::schema::*;
use crate::postgres::*;

pub trait Controller {
    fn get_status(&self) -> i32;
    fn get_id(&self) -> i32;
    fn get_name(&self) -> &str;
    fn get_addr(&self) -> String;
    fn match_pwd(&self, pwd: String) -> bool;
    fn created_at(&self) -> SystemTime;
    fn last_update(&self) -> SystemTime;
}

pub trait Extension {
    fn get_addr(&self) -> String;
}

struct Dummy;
impl Extension for Dummy {
    fn get_addr(&self) -> String {
        "dummy@addres.com".to_string()
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="clients"]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub pwd: String,
    pub status_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[table_name="clients"]
pub struct NewClient<'a> {
    pub name: &'a str,
    pub pwd: &'a str,
    pub status_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}


impl Client {
    pub fn find_by_id(target: i32) -> Result<Box<dyn Controller>, Box<dyn Error>> {
        use crate::schema::clients::dsl::*;
    
        let connection = open_stream();
        let results = clients.filter(id.eq(target))
            .limit(1)
            .load::<Client>(connection)?;
    
        if results.len() > 0 {
            let client = results[0].clone();
            let wrapp = Wrapper::new(client, Box::new(Dummy{}));
            Ok(Box::new(wrapp))
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn create_client<'a>(name: &'a str, pwd: &'a str) -> Result<Box<dyn Controller>, Box<dyn Error>> {
        use crate::schema::clients;

        let new_client = NewClient {
            name: name,
            pwd: pwd,
            status_id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let connection = open_stream();
        let result = diesel::insert_into(clients::table)
            .values(&new_client)
            .get_result::<Client>(connection)?;

        let wrapp = Wrapper::new(result, Box::new(Dummy{}));
        Ok(Box::new(wrapp))
    }
}

// A Wrapper stores the relation between a Client and other structs
struct Wrapper{
    data: Client,
    extension: Box<dyn Extension>,
    creds: Vec<String>,
}

impl Wrapper{
    fn new(data: Client, ext: Box<dyn Extension>) -> Self {
        Wrapper{
            data: data,
            creds: vec!{},
            extension: ext,
        }
    }
}

impl Controller for Wrapper {
    fn get_id(&self) -> i32 {
        self.data.id
    }

    fn get_name(&self) -> &str {
        &self.data.name
    }

    fn get_status(&self) -> i32 {
        self.data.status_id
    }

    fn get_addr(&self) -> String {
        self.extension.get_addr()
    }
    
    fn match_pwd(&self, pwd: String) -> bool {
        self.data.pwd == pwd
    }

    fn created_at(&self) -> SystemTime {
        self.data.created_at
    }

    fn last_update(&self) -> SystemTime {
        self.data.updated_at
    }
}