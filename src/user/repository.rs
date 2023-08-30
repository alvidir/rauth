use super::domain::{Email, Password};
use super::result::{Error, Result};
use super::{application::UserRepository, domain::User};
use async_trait::async_trait;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;

const QUERY_INSERT_USER: &str =
    "INSERT INTO users (name, email, actual_email, password) VALUES ($1, $2, $3, $4) RETURNING id";
const QUERY_INSERT_SALT_SECRET: &str =
    "INSERT INTO secrets (owner, kind, data) VALUES ($1, 'salt', $2)";
const QUERY_FIND_USER: &str = "SELECT u.id, u.email, u.password, s.data FROM users u LEFT JOIN secrets s ON u.id = s.owner WHERE s.id = $1 AND (s.kind = 'salt' OR s IS NULL)";
const QUERY_FIND_USER_BY_EMAIL: &str =
    "SELECT u.id, u.email, u.password, s.data FROM users u LEFT JOIN secrets s ON u.id = s.owner WHERE (u.email = $1 OR u.actual_email = $1) AND (s.kind = 'salt' OR s IS NULL)";
const QUERY_FIND_USER_BY_NAME: &str = "SELECT u.id, u.email, u.password, s.data FROM users u LEFT JOIN secrets s ON u.id - s.owner WHERE u.name = $1 AND (s.kind = 'salt' OR s IS NULL)";
const QUERY_UPDATE_USER: &str =
    "UPDATE users SET name = $1, email = $2, actual_email = $3, password = $4 WHERE id = $5";
const QUERY_DELETE_USER: &str = "DELETE FROM users WHERE id = $1";

// id, email, password, salt
type SelectUserRow = (i32, String, Option<String>, Option<String>);

impl Into<User> for SelectUserRow {
    fn into(self) -> User {
        let mut user = User {
            id: self.0,
            credentials: Email::new(self.1).into(),
        };

        if let (Some(hash), Some(salt)) = (self.2, self.3) {
            user.credentials
                .set_password(Some(Password::new(hash, salt)));
        };

        user
    }
}

pub struct PostgresUserRepository<'a> {
    pub pool: &'a PgPool,
}

macro_rules! on_error {
    ($msg:tt) => {
        |error: sqlx::Error| -> Error {
            if matches!(error, SqlError::RowNotFound) {
                return Error::NotFound;
            }

            error!(error = error.to_string(), $msg,);
            Error::Unknown
        }
    };
}

#[async_trait]
impl<'a> UserRepository for PostgresUserRepository<'a> {
    async fn find(&self, user_id: i32) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER)
            .bind(user_id)
            .fetch_one(self.pool)
            .await
            .map_err(on_error!("performing select user by id query on postgres"))
            .map(SelectUserRow::into)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_EMAIL)
            .bind(email.as_ref())
            .fetch_one(self.pool)
            .await
            .map_err(on_error!(
                "performing select user by email query on postgres"
            ))
            .map(SelectUserRow::into)
    }

    async fn find_by_name(&self, target: &str) -> Result<User> {
        sqlx::query_as(QUERY_FIND_USER_BY_NAME)
            .bind(target)
            .fetch_one(self.pool)
            .await
            .map_err(on_error!(
                "performing select user by name query on postgres"
            ))
            .map(SelectUserRow::into)
    }

    async fn create(&self, user: &mut User) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(|_| Error::Unknown)?;

        let (user_id,) = sqlx::query_as(QUERY_INSERT_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(user.credentials.email.actual_email().as_ref())
            .bind(user.credentials.password.as_ref().map(|pwd| pwd.hash()))
            .fetch_one(&mut *tx)
            .await
            .map_err(on_error!("performing insert user query on postgres"))?;

        if let Some(password) = &user.credentials.password {
            sqlx::query(QUERY_INSERT_SALT_SECRET)
                .bind(user_id)
                .bind(password.salt())
                .execute(&mut *tx)
                .await
                .map_err(on_error!("performing insert salt secret query on postgres"))?;
        }

        tx.commit().await.map_err(|_| Error::Unknown)?;
        user.id = user_id;

        Ok(())
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_UPDATE_USER)
            .bind(user.credentials.email.username())
            .bind(user.credentials.email.as_ref())
            .bind(user.credentials.email.actual_email().as_ref())
            .bind(user.credentials.password.as_ref().map(|pwd| pwd.hash()))
            .bind(user.id)
            .execute(self.pool)
            .await
            .map_err(on_error!("performing update user query on postgres"))?;

        // TODO: Update the password's salt if has changed.
        Ok(())
    }

    async fn delete(&self, user: &User) -> Result<()> {
        sqlx::query(QUERY_DELETE_USER)
            .bind(user.id)
            .execute(self.pool)
            .await
            .map_err(on_error!("performing delete query on postgres"))?;

        Ok(())
    }
}
