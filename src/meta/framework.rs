use std::error::Error;
use std::time::SystemTime;
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::metadata::dsl::*;
use crate::schema::metadata;
use crate::postgres::*;

use super::domain::Metadata;

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(Clone)]
#[table_name = "metadata"]
struct PostgresMetadata {
    pub id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "metadata"]
struct NewPostgresMetadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

pub struct PostgresMetadataRepository {}

impl PostgresMetadataRepository {
    pub fn find(target: i32) -> Result<Metadata, Box<dyn Error>>  {       
        let results = { // block is required because of connection release
            let connection = open_stream().get()?;
            metadata.filter(id.eq(target))
                    .load::<PostgresMetadata>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        Ok(Metadata::new(
            results[0].id,
            results[0].created_at,
            results[0].updated_at,
        ))
    }
}