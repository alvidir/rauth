use std::error::Error;
use std::sync::Arc;
use crate::metadata::application::MetadataRepository;
use crate::constants;
use super::{
    application::SecretRepository,
    domain::Secret,
};
use sqlx::postgres::PgPool;

const QUERY_INSERT_SECRET: &str =
    "INSERT INTO secrets (name, data, user_id, meta_id) VALUES ($1, $2, $3, $4) RETURNING id";
const QUERY_FIND_SECRET: &str = "SELECT id, name, data, user_id, meta_id FROM secrets WHERE id = $1";
const QUERY_FIND_SECRET_BY_USER_AND_NAME: &str =
    "SELECT id, name, data, user_id, meta_id FROM secrets WHERE user_id = $1 AND name = $2";
const QUERY_FIND_SECRET_BY_NAME: &str =
    "SELECT id, name, data, user_id, meta_id FROM secrets WHERE name = $1";
const QUERY_UPDATE_SECRET: &str =
    "UPDATE secrets SET name = $2, data = $3, user_id = $4, meta_id = $5, password = $4 FROM secrets WHERE id = $1";
const QUERY_DELETE_SECRET: &str = "DELETE FROM secrets WHERE id = $1";


type PostgresSecretRow = (i32, String, String, String, i32);

pub struct PostgresSecretRepository<'a, M: MetadataRepository> {
    pub pool: &'a PgPool,
    pub metadata_repo: Arc<M>,
}

impl<'a, M: MetadataRepository> PostgresSecretRepository<'a, M> {
    fn build(&self, secret_row: &PostgresSecretRow) -> Result<Secret, Box<dyn Error>> {
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
            let connection = self.pool.get()
                .map_err(|err| {
                    error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?;

            secrets.filter(id.eq(target))
                .load::<PostgresSecret>(&connection)
                .map_err(|err| {
                    error!("{} performing select query by id on postgres: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?
        };
    
        if results.len() == 0 {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        self.build(&results[0])
    }

    fn find_by_user_and_name(&self, user: i32, secret_name: &str) -> Result<Secret, Box<dyn Error>> {
        use crate::schema::secrets::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()
                .map_err(|err| {
                    error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?;

            secrets.filter(user_id.eq(user))
                .filter(name.eq(secret_name))
                .load::<PostgresSecret>(&connection)
                .map_err(|err| {
                    error!("{} performing select query by user_id and name on postgres: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?
        };
    
        if results.len() == 0 {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        self.build(&results[0])
    }

    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        self.metadata_repo.create(&mut secret.meta)?;

        let row: (i32,) = sqlx::query_as(QUERY_INSERT_SECRET)
            .bind(&secret.name)
            .bind(&secret.data)
            .bind(&secret.user_id)
            .bind(secret.meta.get_id())
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                error!(
                    "{} performing insert query on postgres: {:?}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;

        user.id = row.0;
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
                 
        let connection = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        diesel::update(secrets)
            .filter(id.eq(secret.id))
            .set(&pg_secret)
            .execute(&connection)
            .map_err(|err| {
                error!("{} performing update query on postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for postgres: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        self.tx_delete(&conn, secret)?;
        Ok(())
    }
}