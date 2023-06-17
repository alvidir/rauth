use super::{application::UserRepository, domain::User};
use crate::metadata::application::MetadataRepository;
use crate::result::{Error, Result};
use async_trait::async_trait;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;
use std::sync::Arc;

const QUERY_INSERT_USER: &str =
    "INSERT INTO users (name, email, actual_email, password, meta_id) VALUES ($1, $2, $3, $4, $5) RETURNING id";
const QUERY_FIND_USER: &str =
    "SELECT id, name, email, actual_email, password, meta_id FROM users WHERE id = $1";
const QUERY_FIND_USER_BY_EMAIL: &str =
    "SELECT id, name, email, actual_email, password, meta_id FROM users WHERE email = $1 OR actual_email = $1";
const QUERY_FIND_USER_BY_NAME: &str =
    "SELECT id, name, email, actual_email, password, meta_id FROM users WHERE name = $1";
const QUERY_UPDATE_USER: &str =
    "UPDATE users SET name = $1, email = $2, actual_email = $3, password = $4 WHERE id = $5";
const QUERY_DELETE_USER: &str = "DELETE FROM users WHERE id = $1";

type PostgresUserRow = (i32, String, String, String, String, i32); // id, name, email, actual_email, password, meta_id

pub struct PostgresUserRepository<'a, M: MetadataRepository> {
    pub pool: &'a PgPool,
    pub metadata_repo: Arc<M>,
}

impl<'a, M: MetadataRepository> PostgresUserRepository<'a, M> {
    async fn build(&self, user_raw: &PostgresUserRow) -> Result<User> {
        let meta = self.metadata_repo.find(user_raw.5).await?;

        Ok(User {
            id: user_raw.0,
            name: user_raw.1.clone(),
            email: user_raw.2.clone(),
            actual_email: user_raw.3.clone(),
            password: user_raw.4.clone(),
            meta,
        })
    }
}

#[async_trait]
impl<'a, M: MetadataRepository + std::marker::Sync + std::marker::Send> UserRepository
    for PostgresUserRepository<'a, M>
{
    async fn find(&self, target: i32) -> Result<User> {
        let row: PostgresUserRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_USER)
                .bind(target)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    error!(
                        error = err.to_string(),
                        id = target,
                        "performing select by id query on postgres",
                    );
                    Error::Unknown
                })?
        };

        if row.0 == 0 {
            return Err(Error::NotFound);
        }

        self.build(&row).await // another connection consumed here
    }

    async fn find_by_email(&self, target: &str) -> Result<User> {
        let row: PostgresUserRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
                .bind(target)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    if matches!(err, SqlError::RowNotFound) {
                        return Error::NotFound;
                    }

                    error!(
                        error = err.to_string(),
                        email = target,
                        "performing select by email query on postgres",
                    );
                    Error::Unknown
                })?
        };

        if row.0 == 0 {
            return Err(Error::NotFound);
        }
        self.build(&row).await // another connection consumed here
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        let row: PostgresUserRow = {
            // block is required because of connection release
            sqlx::query_as(QUERY_FIND_USER_BY_NAME)
                .bind(target)
                .fetch_one(self.pool)
                .await
                .map_err(|err| {
                    if matches!(err, SqlError::RowNotFound) {
                        return Error::NotFound;
                    }

                    error!(
                        error = err.to_string(),
                        name = target,
                        "performing select by name query on postgres",
                    );
                    Error::Unknown
                })?
        };

        if row.0 == 0 {
            return Err(Error::NotFound);
        }
        self.build(&row).await // another connection consumed here
    }

    async fn create(&self, user: &mut User) -> Result<()> {
        self.metadata_repo.create(&mut user.meta).await?;

        let row: (i32,) = sqlx::query_as(QUERY_INSERT_USER)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.actual_email)
            .bind(&user.password)
            .bind(user.meta.get_id())
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                error!(
                    error = err.to_string(),
                    "performing insert query on postgres",
                );
                Error::Unknown
            })?;

        user.id = row.0;
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_UPDATE_USER)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.actual_email)
            .bind(&user.password)
            .bind(user.id)
            .execute(self.pool)
            .await
            .map_err(|err| {
                error!(
                    error = err.to_string(),
                    "performing update query on postgres",
                );
                Error::Unknown
            })?;

        Ok(())
    }

    async fn delete(&self, user: &User) -> Result<()> {
        {
            // block is required because of connection release
            sqlx::query(QUERY_DELETE_USER)
                .bind(user.id)
                .execute(self.pool)
                .await
                .map_err(|err| {
                    error!(
                        error = err.to_string(),
                        "performing delete query on postgres",
                    );
                    Error::Unknown
                })?;
        }

        self.metadata_repo.delete(&user.meta).await?; // another connection consumed here
        Ok(())
    }
}
