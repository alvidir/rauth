use std::time::SystemTime;
use std::error::Error;
use diesel::NotFound;
use crate::diesel::prelude::*;
use crate::schema::clients;
use crate::models::enums;
use crate::postgres::*;
use crate::regex::*;

const ERR_UNKNOWN_KIND: &str = "The provided kind do no match with any of the expected ones";

pub trait Ctrl {
    fn get_status(&self) -> i32;
    fn get_id(&self) -> i32;
    fn get_name(&self) -> &str;
    fn get_kind(&self) -> enums::Kind;
    fn created_at(&self) -> SystemTime;
    fn last_update(&self) -> SystemTime;
}

pub fn find_by_id(target: i32) -> Result<Wrapper, Box<dyn Error>> {
    use crate::schema::clients::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        clients.filter(id.eq(target))
        .load::<Client>(&connection)?
    };

    if results.len() > 0 {
        let wrapper = results[0].build()?;
        Ok(wrapper)
    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_name<'a>(target: &'a str) -> Result<Wrapper, Box<dyn Error>>  {
    use crate::schema::clients::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        clients.filter(name.eq(target))
        .load::<Client>(&connection)?
    };

    if results.len() > 0 {
        let client = results[0].clone();
        let build = client.build()?;
        Ok(build)
    } else {
        Err(Box::new(NotFound))
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Identifiable)]
#[derive(Clone)]
#[table_name="clients"]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub status_id: i32,
    pub kind_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[table_name="clients"]
struct NewClient<'a> {
    pub name: &'a str,
    pub status_id: i32,
    pub kind_id: i32,
}

impl Client {
    fn check_kind(kind: i32) -> Result<(), Box<dyn Error>> {
        #![allow(unused)]
        match enums::find_kind_by_id(kind)? {
            enums::Kind::USER | enums::Kind::APP => {
                Ok(())
            }

            _ => {
                Err(ERR_UNKNOWN_KIND.into())
            }
        }
    }

    pub fn new<'a>(kind: i32, name: &'a str) -> Result<Wrapper, Box<dyn Error>> {
        Client::check_kind(kind)?;
        match_name(name)?;

        let client = Client {
            id: 0,
            name: name.to_string(),
            status_id: 1,
            kind_id: kind,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let wrapper = client.build()?;
        Ok(wrapper)
    }

    fn build(&self) -> Result<Wrapper, Box<dyn Error>> {
        let ekind = enums::find_kind_by_id(self.kind_id)?;
        Ok(Wrapper{
            client: self.clone(),
            kind: ekind,
        })
    }
}

pub struct Wrapper {
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

impl super::Gateway for Wrapper {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }
    
    fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        let new_client = NewClient {
            name: &self.client.name,
            status_id: 1,
            kind_id: self.client.kind_id,
        };

        let result = { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::insert_into(clients::table)
            .values(&new_client)
            .get_result::<Client>(&connection)?
        };

        self.client.created_at = result.created_at;
        self.client.updated_at = result.updated_at;
        self.client.id = result.id;
        Ok(())
    }
    
    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.updated_at = SystemTime::now();
        
        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::update(&self.client)
            .set((clients::name.eq(&self.client.name),
                  clients::kind_id.eq(self.client.kind_id),
                  clients::status_id.eq(self.client.status_id),
                  clients::updated_at.eq(self.client.updated_at)))
            .execute(&connection)?;
        }

        Ok(())
    }
    
    fn delete(&self) -> Result<(), Box<dyn Error>> {
        use crate::schema::clients::dsl::*;

        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::delete(
                clients.filter(
                    id.eq(self.get_id())
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}