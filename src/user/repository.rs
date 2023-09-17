use super::domain::{Credentials, Email, PasswordHash, Preferences};
use super::error::{Error, Result};
use super::{application::UserRepository, domain::User};
use crate::base64;
use crate::mfa::domain::MfaMethod;
use crate::on_error;
use crate::postgres::on_query_error;
use crate::secret::domain::SecretKind;
use crate::secret::{domain::Secret, repository::PostgresSecretRepository};
use async_trait::async_trait;
use futures::join;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;
use std::str::FromStr;

const QUERY_INSERT_USER: &str =
    "INSERT INTO users (name, email, actual_email, password, mfa_method) VALUES ($1, $2, $3, $4, $5) RETURNING id";
const QUERY_FIND_USER: &str = "SELECT id, email, password, mfa_method FROM users WHERE id = $1";
const QUERY_FIND_USER_BY_EMAIL: &str =
    "SELECT id, email, password, mfa_method FROM users WHERE email = $1 OR actual_email = $1";
const QUERY_FIND_USER_BY_NAME: &str =
    "SELECT id, email, password, mfa_method FROM users WHERE name = $1";
const QUERY_UPDATE_USER: &str =
    "UPDATE users SET name = $1, email = $2, actual_email = $3, password = $4, mfa_method = $5 WHERE id = $6";
const QUERY_DELETE_USER: &str = "DELETE FROM users WHERE id = $1";

// id, email, password, mfa_method
type SelectUserRow = (i32, String, String, Option<String>);

pub struct PostgresUserRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> PostgresUserRepository<'a> {
    fn construct(row: SelectUserRow, salt: Secret) -> Result<User> {
        Ok(User {
            id: row.0,
            credentials: Credentials {
                email: row.1.try_into()?,
                password: PasswordHash {
                    hash: row.2,
                    salt: base64::encode(salt.data()).try_into()?,
                },
            },
            preferences: Preferences {
                multi_factor: row
                    .3
                    .as_deref()
                    .map(MfaMethod::from_str)
                    .transpose()
                    .map_err(on_error!(Error, "converting string into Mfa value"))?,
            },
        })
    }
}

#[async_trait]
impl<'a> UserRepository for PostgresUserRepository<'a> {
    async fn find(&self, user_id: i32) -> Result<User> {
        let select_user = sqlx::query_as(QUERY_FIND_USER)
            .bind(user_id)
            .fetch_one(self.pool);

        let (secret_result, user_result) = join!(
            PostgresSecretRepository::find_by_owner_and_kind(self.pool, user_id, SecretKind::Salt),
            select_user,
        );

        let salt_secret = secret_result.map_err(Error::from)?;

        let user_row = user_result.map_err(on_query_error!(
            "performing select user by id query on postgres"
        ))?;

        Self::construct(user_row, salt_secret)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User> {
        let user_row: SelectUserRow = sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
            .bind(email.as_ref())
            .fetch_one(self.pool)
            .await
            .map_err(on_query_error!(
                "performing select user by email query on postgres"
            ))?;

        let salt_secret = PostgresSecretRepository::find_by_owner_and_kind(
            self.pool,
            user_row.0,
            SecretKind::Salt,
        )
        .await?;

        Self::construct(user_row, salt_secret)
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        let user_row: SelectUserRow = sqlx::query_as(QUERY_FIND_USER_BY_NAME)
            .bind(target)
            .fetch_one(self.pool)
            .await
            .map_err(on_query_error!(
                "performing select user by name query on postgres"
            ))?;

        let salt_secret = PostgresSecretRepository::find_by_owner_and_kind(
            self.pool,
            user_row.0,
            SecretKind::Salt,
        )
        .await?;

        Self::construct(user_row, salt_secret)
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

        let mut secret = Secret::new(
            SecretKind::Salt,
            user,
            user.credentials.password.salt().as_ref(),
        );

        PostgresSecretRepository::create(&mut *tx, &mut secret).await?;

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
