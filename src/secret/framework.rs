use std::error::Error;
use diesel::NotFound;
use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::secrets;
use crate::schema::secrets::dsl::*;
use crate::metadata::get_repository as get_meta_repository;
use super::domain::{Secret, SecretRepository};

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[table_name = "secrets"]
struct PostgresSecret {
    pub id: i32,
    pub data: String,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "secrets"]
struct NewPostgresSecret<'a> {
    pub data: &'a str,
    pub meta_id: i32,
}

pub struct PostgresSecretRepository;

impl PostgresSecretRepository {
    fn build_first(results: &[PostgresSecret]) -> Result<Secret, Box<dyn Error>> {
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let meta = get_meta_repository().find(results[0].meta_id)?;

        Ok(Secret{
            id: results[0].id,
            data: results[0].data.as_bytes().to_vec(),
            meta: meta,
        })
    }
}

impl SecretRepository for PostgresSecretRepository {
    fn find(&self, target: i32) -> Result<Secret, Box<dyn Error>>  {
        use crate::schema::secrets::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = get_connection().get()?;
            secrets.filter(id.eq(target))
                 .load::<PostgresSecret>(&connection)?
        };
    
        PostgresSecretRepository::build_first(&results)
    }

    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        let new_secret = NewPostgresSecret {
            data: &String::from_utf8(secret.data.clone())?,
            meta_id: secret.meta.get_id(),
        };

        let result = { // block is required because of connection release
            let connection = get_connection().get()?;
            diesel::insert_into(secrets::table)
                .values(&new_secret)
                .get_result::<PostgresSecret>(&connection)?
        };

        secret.id = result.id;
        Ok(())
    }

    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let pg_secret = PostgresSecret {
            id: secret.id,
            data: String::from_utf8(secret.data.clone())?,
            meta_id: secret.meta.get_id(),
        };
        
        { // block is required because of connection release            
            let connection = get_connection().get()?;
            diesel::update(secrets)
                .set(&pg_secret)
                .execute(&connection)?;
        }

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = get_connection().get()?;
            let _result = diesel::delete(
                secrets.filter(
                    id.eq(secret.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}