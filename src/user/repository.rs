use std::sync::Arc;

use super::domain::{Email, Password};
use super::{application::UserRepository, domain::User};
use crate::result::{Error, Result};
use crate::secret::application::SecretRepository;
use crate::secret::domain::{Secret, SecretKind};
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

// id, email, password, salt
type SelectUserRow = (i32, String, Option<String>);

pub struct PostgresUserRepository<'a, M: SecretRepository> {
    pub pool: &'a PgPool,
    pub secret_repo: Arc<M>,
}

impl<'a, M: SecretRepository> PostgresUserRepository<'a, M> {
    async fn user_from_row(&self, (user_id, email, hash): SelectUserRow) -> Result<User> {
        let mut user = User {
            id: user_id,
            credentials: Email::try_from(email)?.into(),
        };

        if let Some(hash) = hash {
            let salt = self
                .secret_repo
                .find_by_owner_and_kind(user_id, SecretKind::Salt)
                .await?;

            user.credentials.password = Some(Password {
                hash: hash.as_bytes().to_vec(),
                salt: salt.data().to_vec(),
            });
        }

        Ok(user)
    }
}

#[async_trait]
impl<'a, M: SecretRepository + std::marker::Sync + std::marker::Send> UserRepository
    for PostgresUserRepository<'a, M>
{
    async fn find(&self, user_id: i32) -> Result<User> {
        let row = sqlx::query_as(QUERY_FIND_USER)
            .bind(user_id)
            .fetch_one(self.pool)
            .await
            .map_err(|err| {
                if matches!(err, SqlError::RowNotFound) {
                    Error::NotFound
                } else {
                    error!(
                        error = err.to_string(),
                        id = user_id,
                        "performing select by id query on postgres",
                    );

                    Error::Unknown
                }
            })?;

        self.user_from_row(row).await
    }

    async fn find_by_email(&self, target: &Email) -> Result<User> {
        let row = sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
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
            })?;

        self.user_from_row(row).await
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        let row = sqlx::query_as(QUERY_FIND_USER_BY_NAME)
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
            })?;

        self.user_from_row(row).await
    }

    async fn create(&self, user: &mut User) -> Result<()> {
        let row: (i32,) = sqlx::query_as(QUERY_INSERT_USER)
            .bind(user.credentials.email.username().to_string())
            .bind(user.credentials.email.as_ref().to_string())
            .bind(
                user.credentials
                    .email
                    .actual_email()
                    .map(|email| email.as_ref().to_string())
                    .unwrap_or(user.credentials.email.as_ref().to_string()),
            )
            .bind(user.credentials.password.as_ref().map(|pwd| pwd.hash()))
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

        if let Some(password) = &user.credentials.password {
            self.secret_repo
                .create(&mut Secret::new(SecretKind::Salt, user, &password.salt))
                .await?;
        }

        Ok(())
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_UPDATE_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(
                user.credentials
                    .email
                    .actual_email()
                    .map(|email| email.as_ref().to_string())
                    .unwrap_or(user.credentials.email.as_ref().to_string()),
            )
            .bind(user.credentials.password.as_ref().map(|pwd| pwd.hash()))
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
        self.secret_repo.delete_by_owner(user).await?;

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

        Ok(())
    }
}
