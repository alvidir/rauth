use super::domain::SecretKind;
use super::error::{Error, Result};
use super::{application::SecretRepository, domain::Secret};
use crate::on_error;
use crate::postgres::on_query_error;
use crate::user::domain::User;
use async_trait::async_trait;
use sqlx::error::Error as SqlError;
use sqlx::postgres::PgPool;
use sqlx::PgExecutor;
use std::str::FromStr;

const QUERY_INSERT_SECRET: &str =
    "INSERT INTO secrets (owner, kind, data) VALUES ($1, $2, $3) RETURNING id";
const QUERY_FIND_SECRET_BY_OWNER_AND_KIND: &str =
    "SELECT id, owner, kind, data FROM secrets WHERE owner = $1 AND kind = $2";
const QUERY_DELETE_SECRET: &str = "DELETE FROM secrets WHERE id = $1";
const QUERY_DELETE_SECRET_BY_OWNER: &str = "DELETE FROM secrets WHERE owner = $1";

type PostgresSecretRow = (i32, i32, String, String); // id, owner, kind, data

impl TryFrom<PostgresSecretRow> for Secret {
    type Error = Error;

    fn try_from(value: PostgresSecretRow) -> Result<Self> {
        Ok(Secret {
            id: value.0,
            owner: value.1.into(),
            kind: SecretKind::from_str(&value.2)
                .map_err(on_error!(Error, "converting string into SecretKind"))?,
            data: value.3.as_bytes().to_vec(),
        })
    }
}

pub struct PostgresSecretRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> PostgresSecretRepository<'a> {
    #[instrument(skip(executor))]
    pub async fn find_by_owner_and_kind<'b, E>(
        executor: E,
        owner: i32,
        kind: SecretKind,
    ) -> Result<Secret>
    where
        E: PgExecutor<'b>,
    {
        sqlx::query_as(QUERY_FIND_SECRET_BY_OWNER_AND_KIND)
            .bind(owner)
            .bind(kind.as_ref())
            .fetch_one(executor)
            .await
            .map_err(on_query_error!(
                "performing select by owner and kind query on postgres"
            ))
            .and_then(PostgresSecretRow::try_into)
    }

    #[instrument(skip(executor))]
    pub async fn create<'b, E>(executor: E, secret: &mut Secret) -> Result<()>
    where
        E: PgExecutor<'b>,
    {
        let row: (i32,) = sqlx::query_as(QUERY_INSERT_SECRET)
            .bind(secret.kind.as_ref())
            .bind(secret.owner)
            .bind(secret.data())
            .fetch_one(executor)
            .await
            .map_err(on_error!(Error, "performing insert query on postgres"))?;

        secret.id = row.0;
        Ok(())
    }

    #[instrument(skip(executor))]
    async fn delete<'b, E>(executor: E, secret: &Secret) -> Result<()>
    where
        E: PgExecutor<'b>,
    {
        sqlx::query(QUERY_DELETE_SECRET)
            .bind(secret.id)
            .fetch_one(executor)
            .await
            .map_err(on_error!(Error, "performing delete query on postgres"))?;

        Ok(())
    }

    #[instrument(skip(executor))]
    async fn delete_by_owner<'b, E>(executor: E, owner: &User) -> Result<()>
    where
        E: PgExecutor<'b>,
    {
        sqlx::query(QUERY_DELETE_SECRET_BY_OWNER)
            .bind(owner.id)
            .fetch_all(executor)
            .await
            .map_err(on_error!(
                Error,
                "performing delete by owner query on postgres"
            ))?;

        Ok(())
    }
}

#[async_trait]
impl<'a> SecretRepository for PostgresSecretRepository<'a> {
    #[instrument(skip(self))]
    async fn find_by_owner_and_kind(&self, owner: i32, kind: SecretKind) -> Result<Secret> {
        Self::find_by_owner_and_kind(self.pool, owner, kind).await
    }

    #[instrument(skip(self))]
    async fn create(&self, secret: &mut Secret) -> Result<()> {
        Self::create(self.pool, secret).await
    }

    #[instrument(skip(self))]
    async fn delete(&self, secret: &Secret) -> Result<()> {
        Self::delete(self.pool, secret).await
    }

    #[instrument(skip(self))]
    async fn delete_by_owner(&self, owner: &User) -> Result<()> {
        Self::delete_by_owner(self.pool, owner).await
    }
}
