use std::error::Error;
use diesel::NotFound;
use crate::schema::kinds;
use crate::diesel::prelude::*;
use crate::postgres::*;

pub const KIND_USER: &str = "USER";
pub const KIND_APP: &str = "APP";

pub trait Controller {
    fn get_id(&self) -> i32;
    fn to_string(&self) -> &str;
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="kinds"]
pub struct Kind {
    pub id: i32,
    pub name: String,
}

impl Kind {
    pub fn find_by_id(target: i32) -> Result<impl Controller, Box<dyn Error>>  {
        use crate::schema::kinds::dsl::*;

        let connection = open_stream();
        let results = kinds.filter(id.eq(target))
            .load::<Kind>(connection)?;

        if results.len() > 0 {
            Ok(results[0].clone())
        } else {
            Err(Box::new(NotFound))
        }
    }

    pub fn find_by_name<'a>(target: &'a str) -> Result<impl Controller, Box<dyn Error>>  {
        use crate::schema::kinds::dsl::*;

        let connection = open_stream();
        let results = kinds.filter(name.eq(target))
            .load::<Kind>(connection)?;

        if results.len() > 0 {
            Ok(results[0].clone())
        } else {
            Err(Box::new(NotFound))
        }
    }
}

impl Controller for Kind {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn to_string(&self) -> &str {
        &self.name
    }

}