use super::{application::MetadataRepository, domain::Metadata};
use crate::constants;
use async_trait::async_trait;
use chrono::naive::NaiveDateTime;
use sqlx::postgres::PgPool;
use std::error::Error;

const QUERY_INSERT_METADATA: &str =
    "INSERT INTO metadata (created_at, updated_at, deleted_at) VALUES ($1, $2, $3) RETURNING id";
const QUERY_FIND_METADATA: &str =
    "SELECT id, created_at, updated_at, deleted_at FROM metadata WHERE id = $1";
const QUERY_UPDATE_METADATA: &str =
    "UPDATE metadata SET created_at = $2, updated_at = $3, deleted_at = $4 FROM metadata WHERE id = $1";
const QUERY_DELETE_METADATA: &str = "DELETE FROM metadata WHERE id = $1";

type PostgresSecretRow = (i32, NaiveDateTime, NaiveDateTime, Option<NaiveDateTime>); // id, created_at, updated_at, deleted_at

pub struct PostgresMetadataRepository<'a> {
    pub pool: &'a PgPool,
}

#[async_trait]
impl<'a> MetadataRepository for PostgresMetadataRepository<'a> {
    async fn find(&self, target: i32) -> Result<Metadata, Box<dyn Error>> {
        let row: PostgresSecretRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_METADATA)
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

        Ok(Metadata {
            id: row.0,
            created_at: row.1,
            updated_at: row.2,
            deleted_at: row.3,
        })
    }

    async fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
        let row: (i32,) = sqlx::query_as(QUERY_INSERT_METADATA)
            .bind(&meta.created_at)
            .bind(&meta.updated_at)
            .bind(&meta.deleted_at)
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

        meta.id = row.0;
        Ok(())
    }

    async fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        sqlx::query(QUERY_UPDATE_METADATA)
            .bind(&meta.id)
            .bind(&meta.created_at)
            .bind(&meta.updated_at)
            .bind(&meta.deleted_at)
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

    async fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
        sqlx::query(QUERY_DELETE_METADATA)
            .bind(&meta.id)
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

        Ok(())
    }
}
