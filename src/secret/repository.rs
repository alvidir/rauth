use std::error::Error;
use std::sync::Arc;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    result::Error as PgError,
    pg::PgConnection,
    NotFound
};

use crate::diesel::prelude::*;
use crate::schema::secrets;
use crate::schema::secrets::dsl::*;
use crate::metadata::application::MetadataRepository;

use super::{
    application::SecretRepository,
    domain::Secret,
};

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[table_name = "secrets"]
struct PostgresSecret {
    pub id: i32,
    pub name: String,
    pub data: String,
    pub user_id: i32,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "secrets"]
struct NewPostgresSecret<'a> {
    pub name: &'a str,
    pub data: &'a str,
    pub user_id: i32,
    pub meta_id: i32,
}

pub struct PostgresSecretRepository<'a, M: MetadataRepository> {
    pub pool: &'a Pool<ConnectionManager<PgConnection>>,
    pub metadata_repo: Arc<M>,
}

impl<'a, M: MetadataRepository> PostgresSecretRepository<'a, M> {
    pub fn tx_create(&self, conn: &PgConnection, secret: &mut Secret) -> Result<(), PgError>  {
        // in order to create a secret it must exists the metadata for this secret
        // PostgresMetadataRepository::tx_create(conn, &mut secret.meta)?;

        let data_as_str = match String::from_utf8(secret.data.clone()) {
            Err(err) => return Err(PgError::DeserializationError(Box::new(err))),
            Ok(data_str) => data_str,
        };
        
        let new_secret = NewPostgresSecret {
            name: &secret.name,
            data: &data_as_str,
            user_id: 0,
            meta_id: secret.meta.get_id(),
        };
        
        let result = diesel::insert_into(secrets::table)
            .values(new_secret)
            .get_result::<PostgresSecret>(conn)?;

        secret.id = result.id;
        Ok(())
    }

    pub fn tx_delete(&self, conn: &PgConnection, secret: &Secret) -> Result<(), PgError>  {
        let _result = diesel::delete(
            secrets.filter(id.eq(secret.id))
        ).execute(conn)?;

        // PostgresMetadataRepository::tx_delete(conn, &secret.meta)?;
        Ok(())
    }

    fn build(&self, pg_secret: &PostgresSecret) -> Result<Secret, Box<dyn Error>> {
        let meta = self.metadata_repo.find(pg_secret.meta_id)?;

        Ok(Secret{
            id: pg_secret.id,
            name: pg_secret.name.clone(),
            data: pg_secret.data.as_bytes().to_vec(),
            meta: meta,
        })
    }
}

impl<'a, M: MetadataRepository> SecretRepository for PostgresSecretRepository<'a, M> {
    fn find(&self, target: i32) -> Result<Secret, Box<dyn Error>>  {
        use crate::schema::secrets::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
            secrets.filter(id.eq(target))
                .load::<PostgresSecret>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        self.build(&results[0])
    }

    fn find_by_user_and_name(&self, user: i32, secret_name: &str) -> Result<Secret, Box<dyn Error>> {
        use crate::schema::secrets::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
            secrets.filter(user_id.eq(user))
                .filter(name.eq(secret_name))
                .load::<PostgresSecret>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        self.build(&results[0])
    }

    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        conn.transaction::<_, PgError, _>(|| self.tx_create(&conn, secret))?;
        Ok(())
    }

    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let pg_secret = PostgresSecret {
            id: secret.id,
            name: secret.name.to_string(),
            data: String::from_utf8(secret.data.clone())?,
            user_id: 0,
            meta_id: secret.meta.get_id(),
        };
                 
        let connection = self.pool.get()?;
        diesel::update(secrets)
            .filter(id.eq(secret.id))
            .set(&pg_secret)
            .execute(&connection)?;

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        self.tx_delete(&conn, secret)?;
        Ok(())
    }
}