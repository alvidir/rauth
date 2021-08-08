use std::error::Error;
use std::time::SystemTime;
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::metadata::dsl::*;
use crate::schema::metadata;
use crate::postgres::*;

use super::domain::{Metadata, MetadataRepository};

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
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

pub struct PostgresMetadataRepository;

impl MetadataRepository for PostgresMetadataRepository {
    fn find(&self, target: i32) -> Result<Metadata, Box<dyn Error>>  {       
        let results = { // block is required because of connection release
            let connection = get_connection().get()?;
            metadata.filter(id.eq(target))
                    .load::<PostgresMetadata>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        Ok(Metadata{
            id: results[0].id,
            created_at: results[0].created_at,
            updated_at: results[0].updated_at,
        })
    }

    fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
        if meta.id == 0 { // create metadata
            let new_meta = NewPostgresMetadata {
                created_at: meta.created_at,
                updated_at: meta.updated_at,
            };
    
            let result = { // block is required because of connection release
                let connection = get_connection().get()?;
                diesel::insert_into(metadata::table)
                    .values(&new_meta)
                    .get_result::<PostgresMetadata>(&connection)?
            };
    
            meta.id = result.id;
            Ok(())

        } else { // update metadata
            let pg_meta = PostgresMetadata {
                id: meta.id,
                created_at: meta.created_at,
                updated_at: meta.updated_at,
            };
            
            { // block is required because of connection release            
                let connection = get_connection().get()?;
                diesel::update(metadata)
                    .set(&pg_meta)
                    .execute(&connection)?;
            }
    
            Ok(())
        }
    }

    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = get_connection().get()?;
            let _result = diesel::delete(
                metadata.filter(
                    id.eq(meta.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }

}