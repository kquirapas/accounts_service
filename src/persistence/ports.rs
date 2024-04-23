use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Connection {
    async fn transaction_start(&mut self) -> Result<()>;
    async fn transaction_commit(&self) -> Result<()>;
    async fn transaction_rollback(&self) -> Result<()>;
}

#[async_trait]
pub trait Actions {
    async fn create_account(&self) -> Result<()>;
    async fn read_account(&self) -> Result<()>;
    async fn update_account(&self) -> Result<()>;
    async fn delete_account(&self) -> Result<()>;
}

/// Sub trait of super traits Adapter and Actions
/// to guarantee the existence of both.
pub trait Persistence: Connection + Actions {}
