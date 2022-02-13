use std::error::Error;
use std::time::SystemTime;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection,
};

use crate::diesel::prelude::*;
use crate::schema::metadata::dsl::*;
use crate::schema::metadata;
use crate::constants;

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

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub struct PostgresMetadataRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> PostgresMetadataRepository<'a> {
    pub fn tx_create(conn: &PgConnection, meta: &mut Metadata) -> Result<(), Box<dyn Error>>  {
        let new_meta = NewPostgresMetadata {
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            deleted_at: None,
        };
        
        let result = diesel::insert_into(metadata::table)
            .values(new_meta)
            .get_result::<PostgresMetadata>(conn)
            .map_err(|err| {
                error!("{} performing insert query on postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        meta.id = result.id;
        Ok(())
    }

    pub fn tx_delete(conn: &PgConnection, meta: &Metadata) -> Result<(), Box<dyn Error>>  {
        diesel::delete(metadata.filter(id.eq(meta.id)))
            .execute(conn)
            .map_err(|err| {
                error!("{} performing delete query on postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        Ok(())
    }
}

impl<'a> MetadataRepository for PostgresMetadataRepository<'a> {
    fn find(&self, target: i32) -> Result<Metadata, Box<dyn Error>>  {       
        let results = { // block is required because of connection release
            let connection = self.pool.get()
                .map_err(|err| {
                    error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?;

            metadata.filter(id.eq(target))
                    .load::<PostgresMetadata>(&connection)
                    .map_err(|err| {
                        error!("{} performing select by id query on postgres: {}", constants::ERR_UNKNOWN, err);
                        constants::ERR_UNKNOWN
                    })?
        };
    
        if results.len() == 0 {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        Ok(Metadata{
            id: results[0].id,
            created_at: results[0].created_at,
            updated_at: results[0].updated_at,
            deleted_at: results[0].deleted_at,
        })
    }

    fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

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
                 
        let connection = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
        
        diesel::update(metadata)
            .filter(id.eq(meta.id))
            .set(&pg_meta)
            .execute(&connection)
            .map_err(|err| {
                error!("{} performing update query on postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        Ok(())
    }

    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
        
        PostgresMetadataRepository::tx_delete(&conn, meta)?;
        Ok(())
    }

}