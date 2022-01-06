use std::error::Error;
use std::time::SystemTime;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    result::Error as PgError,
    pg::PgConnection,
    NotFound
};

use crate::diesel::prelude::*;
use crate::schema::metadata::dsl::*;
use crate::schema::metadata;

use super::{
    domain::Metadata,
    application::MetadataRepository
};

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[table_name = "metadata"]
struct PostgresMetadata {
    pub id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub deleted_at: Option<SystemTime>
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "metadata"]
struct NewPostgresMetadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub deleted_at: Option<SystemTime>
}

pub struct PostgresMetadataRepository<'a> {
    pub pool: &'a Pool<ConnectionManager<PgConnection>>,
}

impl<'a> PostgresMetadataRepository<'a> {
    pub fn tx_create(conn: &PgConnection, meta: &mut Metadata) -> Result<(), PgError>  {
        let new_meta = NewPostgresMetadata {
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            deleted_at: None,
        };
        
        let result = diesel::insert_into(metadata::table)
            .values(new_meta)
            .get_result::<PostgresMetadata>(conn)?;

        meta.id = result.id;
        Ok(())
    }

    pub fn tx_delete(conn: &PgConnection, meta: &Metadata) -> Result<(), PgError>  {
        diesel::delete(
            metadata.filter(id.eq(meta.id))
        ).execute(conn)?;

        Ok(())
    }
}

impl<'a> MetadataRepository for PostgresMetadataRepository<'a> {
    fn find(&self, target: i32) -> Result<Metadata, Box<dyn Error>>  {       
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
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
            deleted_at: results[0].deleted_at,
        })
    }

    fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        PostgresMetadataRepository::tx_create(&conn, meta)?;
        Ok(())
    }

    fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        let pg_meta = PostgresMetadata {
            id: meta.id,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            deleted_at: meta.deleted_at,
        };
                 
        let connection = self.pool.get()?;
        diesel::update(metadata)
            .filter(id.eq(meta.id))
            .set(&pg_meta)
            .execute(&connection)?;

        Ok(())
    }

    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        PostgresMetadataRepository::tx_delete(&conn, meta)?;
        Ok(())
    }

}