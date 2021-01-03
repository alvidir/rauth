pub mod app;
pub mod user;

use self::user::*;
use self::app::*;

use diesel::NotFound;
use std::error::Error;
use std::time::SystemTime;
use crate::diesel::prelude::*;
use crate::schema::clients;
use crate::models::kind::{Kind, Controller as KindController, KIND_USER, KIND_APP};
use crate::postgres::*;
use crate::regex::*;

const ERR_UNKNOWN_KIND: &str = "The provided kind do no match with any of the expected ones";
const ERR_EXTENSION_EXISTS: &str = "The current client already has an extension";

pub trait Controller {
    fn get_status(&self) -> i32;
    fn get_id(&self) -> i32;
    fn get_name(&self) -> &str;
    fn get_addr(&self) -> String;
    fn match_pwd(&self, pwd: String) -> bool;
    fn created_at(&self) -> SystemTime;
    fn last_update(&self) -> SystemTime;
    fn set_extension(&mut self, ext: Box<dyn Extension>) -> Result<(), String>;
}

pub trait Extension {
    fn get_addr(&self) -> String;
    fn get_client_id(&self) -> i32;
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
    pub kind_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[table_name="clients"]
pub struct NewClient<'a> {
    pub name: &'a str,
    pub pwd: &'a str,
    pub status_id: i32,
    pub kind_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Client {
    fn check_kind(kind: i32) -> Result<(), Box<dyn Error>> {
        match Kind::find_by_id(kind)?.to_string() {
            KIND_USER | KIND_APP => {
                Ok(())
            }

            _ => {
                Err(ERR_UNKNOWN_KIND.into())
            }
        }
    }

    pub fn create<'a>(kind: i32, name: &'a str, pwd: &'a str) -> Result<Box<dyn Controller>, Box<dyn Error>> {
        Client::check_kind(kind)?;
        match_name(name)?;
        match_pwd(pwd)?;

        let new_client = NewClient {
            name: name,
            pwd: pwd,
            status_id: 0,
            kind_id: kind,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let connection = open_stream();
        let result = diesel::insert_into(clients::table)
            .values(&new_client)
            .get_result::<Client>(connection)?;

        let wrapp = Wrapper::new(result, None);
        Ok(Box::new(wrapp))
    }
    
    pub fn find_by_id(target: i32) -> Result<Box<dyn Controller>, Box<dyn Error>> {
        use crate::schema::clients::dsl::*;
    
        let connection = open_stream();
        let results = clients.filter(id.eq(target))
            .load::<Client>(connection)?;
    
        if results.len() > 0 {
            let client = results[0].clone();
            let extension = client.find_extension_by_client_id()?;
            let wrapp = Wrapper::new(client, Some(extension));
            Ok(Box::new(wrapp))
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn find_by_name<'a>(target: &'a str) -> Result<Box<dyn Controller>, Box<dyn Error>>  {
        use crate::schema::clients::dsl::*;

        let connection = open_stream();
        let results = clients.filter(name.eq(target))
            .load::<Client>(connection)?;

        if results.len() > 0 {
            let client = results[0].clone();
            let extension = client.find_extension_by_client_id()?;
            let wrapp = Wrapper::new(client, Some(extension));
            Ok(Box::new(wrapp))
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn from_extension(ext: Box<dyn Extension>) -> Result<Box<dyn Controller>, Box<dyn Error>> {
        use crate::schema::clients::dsl::*;
    
        let connection = open_stream();
        let results = clients.filter(id.eq(ext.get_client_id()))
            .load::<Client>(connection)?;
    
        if results.len() > 0 {
            let client = results[0].clone();
            let wrapp = Wrapper::new(client, Some(ext));
            Ok(Box::new(wrapp))
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn find_extension_by_client_id(&self) -> Result<Box<dyn Extension>, Box<dyn Error>> {
        let kind = Kind::find_by_id(self.kind_id)?;
        match kind.to_string() {
            KIND_USER => {
                let user = User::find_by_client_id(self.id)?;
                Ok(Box::new(user))
            }

            KIND_APP => {
                let app = App::find_by_client_id(self.id)?;
                Ok(Box::new(app))
            }

            _ => {
                Err(ERR_UNKNOWN_KIND.into())
            }
        }
    }
}

// A Wrapper stores the relation between a Client and other structs
struct Wrapper{
    data: Client,
    extension: Option<Box<dyn Extension>>,
    _creds: Vec<String>,
}

impl Wrapper{
    fn new(data: Client, ext: Option<Box<dyn Extension>>) -> Self {
        Wrapper{
            data: data,
            _creds: vec!{},
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
        match &self.extension {
            None => {
                "".to_string()
            }

            Some(extension) => {
                extension.get_addr()
            }
        }
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

    fn set_extension(&mut self, ext: Box<dyn Extension>) -> Result<(), String> {
        match self.extension {
            None => {
                self.extension = Some(ext);
                Ok(())
            }

            Some(_) => {
                let msg = format!("{}", ERR_EXTENSION_EXISTS);
                Err(msg)
            }
        }
    }
}