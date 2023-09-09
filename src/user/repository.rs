use std::str::FromStr;

use super::domain::{Credentials, Email, PasswordHash, Preferences};
use super::error::{Error, Result};
use super::{application::UserRepository, domain::User};
use crate::mfa::domain::MfaMethod;
use crate::on_error;
use crate::postgres::on_query_error;
use async_trait::async_trait;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;

const QUERY_INSERT_USER: &str =
    "INSERT INTO users (name, email, actual_email, password, multi_factor) VALUES ($1, $2, $3, $4, $5) RETURNING id";
const QUERY_INSERT_SALT_SECRET: &str =
    "INSERT INTO secrets (owner, kind, data) VALUES ($1, 'salt', $2)";
const QUERY_FIND_USER: &str = "SELECT u.id, u.email, u.password, s.data, u.multi_factor FROM users u LEFT JOIN secrets s ON u.id = s.owner WHERE s.id = $1 AND s.kind = 'salt'";
const QUERY_FIND_USER_BY_EMAIL: &str =
    "SELECT u.id, u.email, u.password, s.data, u.multi_factor FROM users u LEFT JOIN secrets s ON u.id = s.owner WHERE (u.email = $1 OR u.actual_email = $1) AND s.kind = 'salt'";
const QUERY_FIND_USER_BY_NAME: &str = "SELECT u.id, u.email, u.password, s.data, u.multi_factor FROM users u LEFT JOIN secrets s ON u.id - s.owner WHERE u.name = $1 AND s.kind = 'salt'";
const QUERY_UPDATE_USER: &str =
    "UPDATE users SET name = $1, email = $2, actual_email = $3, password = $4, multi_factor = $5 WHERE id = $6";
const QUERY_DELETE_USER: &str = "DELETE FROM users WHERE id = $1";

// id, email, password, salt, multi_factor
type SelectUserRow = (i32, String, String, String, Option<String>);

impl TryInto<User> for SelectUserRow {
    type Error = Error;

    fn try_into(self) -> Result<User> {
        Ok(User {
            id: self.0,
            credentials: Credentials {
                email: self.1.try_into()?,
                password: PasswordHash {
                    hash: self.2,
                    salt: self.3,
                },
            },
            preferences: Preferences {
                multi_factor: self
                    .4
                    .as_deref()
                    .map(MfaMethod::from_str)
                    .transpose()
                    .map_err(on_error!(Error, "converting string into Mfa value"))?,
            },
        })
    }
}

pub struct PostgresUserRepository<'a> {
    pub pool: &'a PgPool,
}

#[async_trait]
impl<'a> UserRepository for PostgresUserRepository<'a> {
    async fn find(&self, user_id: i32) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER)
            .bind(user_id)
            .fetch_one(self.pool)
            .await
            .map_err(on_query_error!(
                "performing select user by id query on postgres"
            ))
            .and_then(SelectUserRow::try_into)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
            .bind(email.as_ref())
            .fetch_one(self.pool)
            .await
            .map_err(on_query_error!(
                "performing select user by email query on postgres"
            ))
            .and_then(SelectUserRow::try_into)
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_NAME)
            .bind(target)
            .fetch_one(self.pool)
            .await
            .map_err(on_query_error!(
                "performing select user by name query on postgres"
            ))
            .and_then(SelectUserRow::try_into)
    }

    async fn create(&self, user: &mut User) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(on_error!(Error, "starting postgres transaction"))?;

        let (user_id,) = sqlx::query_as(QUERY_INSERT_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(user.credentials.email.actual_email().as_ref())
            .bind(user.credentials.password.as_ref())
            .bind(
                user.preferences
                    .multi_factor
                    .as_ref()
                    .map(ToString::to_string),
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(on_error!(Error, "performing insert user query on postgres"))?;

        sqlx::query(QUERY_INSERT_SALT_SECRET)
            .bind(user_id)
            .bind(user.credentials.password.salt())
            .execute(&mut *tx)
            .await
            .map_err(on_error!(
                Error,
                "performing insert salt secret query on postgres"
            ))?;

        tx.commit()
            .await
            .map_err(on_error!(Error, "commiting postgres transaction"))?;

        user.id = user_id;
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_UPDATE_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(user.credentials.email.actual_email().as_ref())
            .bind(user.credentials.password.as_ref())
            .bind(
                user.preferences
                    .multi_factor
                    .as_ref()
                    .map(ToString::to_string),
            )
            .bind(user.id)
            .execute(self.pool)
            .await
            .map_err(on_error!(Error, "performing update user query on postgres"))?;

        // TODO: Update the password's salt if has changed.
        Ok(())
    }

    async fn delete(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_DELETE_USER)
            .bind(user.id)
            .execute(self.pool)
            .await
            .map_err(on_error!(Error, "performing delete query on postgres"))?;

        Ok(())
    }
}
