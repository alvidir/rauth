use std::time::SystemTime;
use std::error::Error;
use diesel::NotFound;
use crate::diesel::prelude::*;
use crate::schema::clients;
use crate::models::enums;
use crate::postgres::*;
use crate::regex::*;

const ERR_UNKNOWN_KIND: &str = "The provided kind do no match with any of the expected ones";
const ERR_EXTENSION_EXISTS: &str = "The current client already has an extension";

pub trait Ctrl {
    fn get_status(&self) -> i32;
    fn get_id(&self) -> i32;
    fn get_name(&self) -> &str;
    fn get_addr(&self) -> &str;
    fn get_kind(&self) -> enums::Kind;
    fn created_at(&self) -> SystemTime;
    fn last_update(&self) -> SystemTime;
}

pub fn find_by_id(target: i32) -> Result<Box<dyn Ctrl>, Box<dyn Error>> {
    use crate::schema::clients::dsl::*;

    let connection = open_stream();
    let results = clients.filter(id.eq(target))
        .load::<Client>(connection)?;

    if results.len() > 0 {
        let wrapper = results[0].build()?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_name<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::clients::dsl::*;

    let connection = open_stream();
    let results = clients.filter(name.eq(target))
        .load::<Client>(connection)?;

    if results.len() > 0 {
        let client = results[0].clone();
        let build = client.build()?;
        Ok(Box::new(build))
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_addr<'a>(target: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::clients::dsl::*;

    let connection = open_stream();
    let results = clients.filter(address.eq(target))
        .load::<Client>(connection)?;

    if results.len() > 0 {
        let client = results[0].clone();
        let build = client.build()?;
        Ok(Box::new(build))
    } else {
        Err(Box::new(NotFound))
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="clients"]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub status_id: i32,
    pub kind_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[table_name="clients"]
struct NewClient<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub status_id: i32,
    pub kind_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Client {
    fn check_kind(kind: i32) -> Result<(), Box<dyn Error>> {
        match enums::find_kind_by_id(kind)? {
            enums::Kind::USER | enums::Kind::APP => {
                Ok(())
            }

            _ => {
                Err(ERR_UNKNOWN_KIND.into())
            }
        }
    }

    pub fn new<'a>(kind: i32, name: &'a str, address: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>> {
        Client::check_kind(kind)?;
        match_name(name)?;

        let new_client = NewClient {
            name: name,
            address: address,
            status_id: 1,
            kind_id: kind,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let connection = open_stream();
        let result = diesel::insert_into(clients::table)
            .values(&new_client)
            .get_result::<Client>(connection)?;

        let wrapper = result.build()?;
        Ok(Box::new(wrapper))
    }

    fn build(&self) -> Result<Wrapper, Box<dyn Error>> {
        let ekind = enums::find_kind_by_id(self.kind_id)?;
        Ok(Wrapper{
            client: self.clone(),
            kind: ekind,
        })
    }
}

struct Wrapper {
    client: Client,
    kind: enums::Kind,
}

impl Ctrl for Wrapper {
    fn get_status(&self) -> i32 {
        self.client.status_id
    }

    fn get_id(&self) -> i32 {
        self.client.id
    }

    fn get_name(&self) -> &str {
        &self.client.name
    }

    fn get_addr(&self) -> &str {
        &self.client.address
    }

    fn get_kind(&self) -> enums::Kind {
        self.kind
    }

    fn created_at(&self) -> SystemTime {
        self.client.created_at
    }

    fn last_update(&self) -> SystemTime {
        self.client.updated_at
    }
}