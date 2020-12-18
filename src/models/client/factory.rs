use std::error::Error;
use std::time::SystemTime;
use crate::models::client::{Client, Controller as ClientController, *};
use crate::diesel::prelude::*;
use crate::schema::*;
use crate::postgres::*;

#[derive(Insertable)]
#[derive(Queryable)]
#[table_name="clients"]
pub struct Gateway {
    pub id: i32,
    pub name: String,
    pub pwd: String,
    pub status_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Gateway {
    pub fn new(name: String, pwd: String) -> Self {
        Gateway {
            id: 0,
            name: name,
            pwd: pwd,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            status_id: 0,
        }
    }

    //pub fn build(&self) -> Box<dyn ClientController> {
    //    let client: Client = Client::build(, self);
    //    
    //}
}



pub fn get_client_by_id(target: i32) -> Result<Option<Box<dyn ClientController>>, Box<dyn Error>> {
    use crate::schema::clients::dsl::*;

    let connection = open_stream();
    let results = clients.filter(id.eq(target))
        .limit(1)
        .load::<Gateway>(connection)?;

    if results.len() > 0 {
        Ok(None)
    } else {
        Ok(None)
    }
}