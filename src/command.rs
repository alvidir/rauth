use async_trait::async_trait;

#[async_trait]
pub trait Command {
    type Result;

    /// Executes the corresponding command, returning the associated result.
    async fn execute(self) -> Self::Result;
}
