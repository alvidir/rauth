use super::{application::SecretRepository, domain::Secret};
use crate::constants;
use crate::metadata::application::MetadataRepository;
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::error::Error;
use std::sync::Arc;

const QUERY_INSERT_SECRET: &str =
    "INSERT INTO secrets (name, data, user_id, meta_id) VALUES ($1, $2, $3, $4) RETURNING id";
const QUERY_FIND_SECRET: &str =
    "SELECT id, name, data, user_id, meta_id FROM secrets WHERE id = $1";
const QUERY_FIND_SECRET_BY_USER_AND_NAME: &str =
    "SELECT id, name, data, user_id, meta_id FROM secrets WHERE user_id = $1 AND name = $2";
const QUERY_UPDATE_SECRET: &str =
    "UPDATE secrets SET name = $2, data = $3, user_id = $4, meta_id = $5, password = $4 FROM secrets WHERE id = $1";
const QUERY_DELETE_SECRET: &str = "DELETE FROM secrets WHERE id = $1";

type PostgresSecretRow = (i32, String, String, i32, i32); // id, name, data, user_id, meta_id

pub struct PostgresSecretRepository<'a, M: MetadataRepository> {
    pub pool: &'a PgPool,
    pub metadata_repo: Arc<M>,
}

impl<'a, M: MetadataRepository> PostgresSecretRepository<'a, M> {
    async fn build(&self, secret_row: &PostgresSecretRow) -> Result<Secret, Box<dyn Error>> {
        let meta = self.metadata_repo.find(secret_row.4).await?;

        Ok(Secret {
            id: secret_row.0,
            name: secret_row.1.clone(),
            data: secret_row.2.as_bytes().to_vec(),
            owner: secret_row.3,
            meta,
        })
    }
}

#[async_trait]
impl<'a, M: MetadataRepository + std::marker::Sync + std::marker::Send> SecretRepository
    for PostgresSecretRepository<'a, M>
{
    async fn find(&self, target: i32) -> Result<Secret, Box<dyn Error>> {
        let row: PostgresSecretRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_SECRET)
                .bind(target)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    error!(
                        "{} performing select by id query on postgres: {:?}",
                        constants::ERR_UNKNOWN,
                        err
                    );
                    constants::ERR_UNKNOWN
                })?
        };

        if row.0 == 0 {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        self.build(&row).await // another connection consumed here
    }

    async fn find_by_user_and_name(
        &self,
        user: i32,
        secret_name: &str,
    ) -> Result<Secret, Box<dyn Error>> {
        let row: PostgresSecretRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_SECRET_BY_USER_AND_NAME)
                .bind(user)
                .bind(secret_name)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    error!(
                        "{} performing select by user and name query on postgres: {:?}",
                        constants::ERR_UNKNOWN,
                        err
                    );
                    constants::ERR_UNKNOWN
                })?
        };

        if row.0 == 0 {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        self.build(&row).await // another connection consumed here
    }

    async fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        self.metadata_repo.create(&mut secret.meta).await?;

        let row: (i32,) = sqlx::query_as(QUERY_INSERT_SECRET)
            .bind(&secret.name)
            .bind(&secret.data)
            .bind(&secret.owner)
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

        secret.id = row.0;
        Ok(())
    }

    async fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        sqlx::query(QUERY_UPDATE_SECRET)
            .bind(&secret.id)
            .bind(&secret.name)
            .bind(&secret.data)
            .bind(&secret.owner)
            .bind(&secret.meta.get_id())
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                error!(
                    "{} performing update query on postgres: {:?}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;

        Ok(())
    }

    async fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        {
            // block is required because of connection release
            sqlx::query(QUERY_DELETE_SECRET)
                .bind(&secret.id)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    error!(
                        "{} performing delete query on postgres: {:?}",
                        constants::ERR_UNKNOWN,
                        err
                    );
                    constants::ERR_UNKNOWN
                })?;
        }

        self.metadata_repo.delete(&secret.meta).await?; // another connection consumed here
        Ok(())
    }
}
