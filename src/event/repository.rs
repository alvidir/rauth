use super::{
    domain::Event,
    error::{Error, Result},
    service::EventRepository,
};
use crate::macros::on_error;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};

const QUERY_INSERT_EVENT: &str = "INSERT INTO events (checksum, payload) VALUES ($1, $2)";
const QUERY_SELECT_EVENTS: &str =
    "SELECT checksum, payload FROM events ORDER BY created_at ASC LIMIT $1";
const QUERY_DELETE_EVENT: &str = "DELETE FROM events WHERE id = $1";

// checksum, payload
type PostgresEventRow = (String, String);

pub struct PostgresEventRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> PostgresEventRepository<'a> {
    #[instrument(skip(executor))]
    pub async fn create<'b, E>(executor: E, event: &Event) -> Result<()>
    where
        E: PgExecutor<'b>,
    {
        sqlx::query(QUERY_INSERT_EVENT)
            .bind(event.id.as_str())
            .bind(event.payload.as_str())
            .fetch_one(executor)
            .await
            .map_err(on_error!(Error, "performing insert query on postgres"))?;

        Ok(())
    }

    #[instrument(skip(executor))]
    async fn delete<'b, E>(executor: E, event: &Event) -> Result<()>
    where
        E: PgExecutor<'b>,
    {
        sqlx::query(QUERY_DELETE_EVENT)
            .bind(event.id.as_str())
            .fetch_one(executor)
            .await
            .map_err(on_error!(Error, "performing delete query on postgres"))?;

        Ok(())
    }
}

#[async_trait]
impl<'a> EventRepository for PostgresEventRepository<'a> {
    #[instrument(skip(self))]
    async fn list(&self, limit: usize) -> Result<Vec<Event>> {
        let result: Vec<PostgresEventRow> = sqlx::query_as(QUERY_SELECT_EVENTS)
            .bind(limit as i32)
            .fetch_all(self.pool)
            .await
            .map_err(on_error!(Error, "performing select query on postgres"))?;

        Ok(result
            .into_iter()
            .map(|row| Event {
                id: row.0,
                payload: row.1,
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, event: &Event) -> Result<()> {
        Self::create(self.pool, event).await
    }

    #[instrument(skip(self))]
    async fn delete(&self, event: &Event) -> Result<()> {
        Self::delete(self.pool, event).await
    }
}
