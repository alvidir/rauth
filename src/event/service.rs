use super::{domain::Event, error::Result};
use async_trait::async_trait;

#[async_trait]
pub trait EventRepository {
    async fn list(&self, limit: usize) -> Result<Vec<Event>>;
    async fn create(&self, event: &Event) -> Result<()>;
    async fn delete(&self, event: &Event) -> Result<()>;
}

#[async_trait]
pub trait EventService {
    async fn emit(&self, event: &Event) -> Result<()>;
}
