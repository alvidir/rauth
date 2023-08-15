use super::domain::Email;
use super::{application::UserRepository, domain::User};
use crate::result::{Error, Result};
use async_trait::async_trait;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;

const QUERY_INSERT_USER: &str =
    "INSERT INTO users (name, email, actual_email, password) VALUES ($1, $2, $3, $4) RETURNING id";
const QUERY_FIND_USER: &str = "SELECT id, email, password FROM users WHERE id = $1";
const QUERY_FIND_USER_BY_EMAIL: &str =
    "SELECT id, email, password FROM users WHERE email = $1 OR actual_email = $1";
const QUERY_FIND_USER_BY_NAME: &str = "SELECT id, email, password FROM users WHERE name = $1";
const QUERY_UPDATE_USER: &str =
    "UPDATE users SET name = $1, email = $2, actual_email = $3, password = $4 WHERE id = $5";
const QUERY_DELETE_USER: &str = "DELETE FROM users WHERE id = $1";

type PostgresUserRow = (i32, String, String); // id, email, password

impl TryFrom<PostgresUserRow> for User {
    type Error = Error;

    fn try_from(value: PostgresUserRow) -> Result<Self> {
        Ok(User {
            id: value.0,
            credentials: (value.1.as_str(), value.2.as_str()).try_into()?,
        })
    }
}

pub struct PostgresUserRepository<'a> {
    pub pool: &'a PgPool,
}

#[async_trait]
impl<'a> UserRepository for PostgresUserRepository<'a> {
    async fn find(&self, target: i32) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER)
            .bind(target)
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                if matches!(err, SqlError::RowNotFound) {
                    Error::NotFound
                } else {
                    error!(
                        error = err.to_string(),
                        id = target,
                        "performing select by id query on postgres",
                    );

                    Error::Unknown
                }
            })
            .and_then(PostgresUserRow::try_into)
    }

    async fn find_by_email(&self, target: &Email) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
            .bind(target.as_ref())
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                if matches!(err, SqlError::RowNotFound) {
                    Error::NotFound
                } else {
                    error!(
                        error = err.to_string(),
                        email = target.as_ref(),
                        "performing select by email query on postgres",
                    );

                    Error::Unknown
                }
            })
            .and_then(PostgresUserRow::try_into)
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_NAME)
            .bind(target)
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                if matches!(err, SqlError::RowNotFound) {
                    Error::NotFound
                } else {
                    error!(
                        error = err.to_string(),
                        name = target,
                        "performing select by name query on postgres",
                    );

                    Error::Unknown
                }
            })
            .and_then(PostgresUserRow::try_into)
    }

    async fn create(&self, user: &mut User) -> Result<()> {
        let row: (i32,) = sqlx::query_as(QUERY_INSERT_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(
                user.credentials
                    .email
                    .actual_email()
                    .unwrap_or(user.credentials.email)
                    .as_ref(),
            )
            .bind(user.credentials.password.ok_or(Error::Unknown)?)
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                error!(
                    error = err.to_string(),
                    "performing insert query on postgres",
                );
                Error::Unknown
            })
            .and_then(PostgresUserRow::try_into)?;

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
