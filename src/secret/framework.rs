use std::error::Error;
use diesel::NotFound;
use diesel::result::Error as PgError;

use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::secrets;
use crate::schema::secrets::dsl::*;
use crate::metadata::{
    get_repository as get_meta_repository,
    framework::PostgresMetadataRepository,
};
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
    pub fn create_on_conn(conn: &PgConnection, secret: &mut Secret) -> Result<(), PgError>  {
        // in order to create a secret it must exists the metadata for this secret
        PostgresMetadataRepository::create_on_conn(conn, &mut secret.meta)?;

        let data_as_str = match String::from_utf8(secret.data.clone()) {
            Err(err) => return Err(PgError::DeserializationError(Box::new(err))),
            Ok(data_str) => data_str,
        };
        
        let new_secret = NewPostgresSecret {
            data: &data_as_str,
            meta_id: secret.meta.get_id(),
        };
        
        let result = diesel::insert_into(secrets::table)
            .values(new_secret)
            .get_result::<PostgresSecret>(conn)?;

        secret.id = result.id;
        Ok(())
    }

    pub fn delete_on_conn(conn: &PgConnection, secret: &Secret) -> Result<(), PgError>  {
        let _result = diesel::delete(
            secrets.filter(id.eq(secret.id))
        ).execute(conn)?;

        PostgresMetadataRepository::delete_on_conn(conn, &secret.meta)?;
        Ok(())
    }

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
        let conn = get_connection().get()?;
        conn.transaction::<_, PgError, _>(|| PostgresSecretRepository::create_on_conn(&conn, secret))?;
        Ok(())
    }

    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let pg_secret = PostgresSecret {
            id: secret.id,
            data: String::from_utf8(secret.data.clone())?,
            meta_id: secret.meta.get_id(),
        };
                 
        let connection = get_connection().get()?;
        diesel::update(secrets)
            .filter(id.eq(secret.id))
            .set(&pg_secret)
            .execute(&connection)?;

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let conn = get_connection().get()?;
        PostgresSecretRepository::delete_on_conn(&conn, secret)?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
pub mod tests {
    use super::super::{
        domain::Secret,
        get_repository as get_secret_repository,
    };

    #[test]
    fn secret_insert_should_not_fail() {
        let mut secret = Secret::new("secret_insert_should_success".as_bytes());
        get_secret_repository().create(&mut secret).unwrap();

        assert_eq!("secret_insert_should_success".as_bytes(), secret.data);
        get_secret_repository().delete(&secret).unwrap();
    }

    #[test]
    fn secret_save_modified_data_should_fail() {
        let mut secret = Secret::new("secret_save_modified_data_should_fail".as_bytes());
        get_secret_repository().create(&mut secret).unwrap();

        secret.data = "secret_save_modified_data_should_fail_2".as_bytes().to_vec();
        assert!(get_secret_repository().save(&secret).is_err());
        get_secret_repository().delete(&secret).unwrap();
    }
}