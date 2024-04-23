use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use sqlx::{
    postgres::{PgPool, PgPoolOptions, Postgres},
    Transaction,
};

use crate::persistence::ports::Persistence;

pub struct PostgresPersistence<'a> {
    pub pool: PgPool,
    pub transaction: Arc<Mutex<Option<Transaction<'a, Postgres>>>>,
}

impl PostgresPersistence<'_> {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .max_connections(5)
                .connect("postgres://superuser:superpassword@0.0.0.0:5432/postgres")
                .await
                .with_context(|| "Failed to connect to Postgres DB")?,
            transaction: Arc::new(Mutex::new(None)),
        })
    }
}

impl Persistence for PostgresPersistence<'_> {}

#[async_trait]
impl Connection for PostgresPersistence<'_> {
    async fn transaction_start(&mut self) -> Result<()> {
        let shared_tx_ref = Arc::clone(&self.transaction);

        println!("before self.transaction: {:?}", self.transaction);
        let new_tx = self.pool.begin().await?;

        // limit lock acquisition in block scope
        {
            // acquiring lock
            let mut lock = shared_tx_ref.lock().unwrap();
            // dereferencing MutexGuard (impl DerefMut)
            let tx_option = lock.deref_mut();
            // Ensure no other Transactions already started
            ensure!(
                tx_option.is_none(),
                ConnectionError::TransactionAlreadyRunning
            );

            *tx_option = Some(new_tx)
        };

        Ok(())
    }

    async fn transaction_commit(&self) -> Result<()> {
        let shared_tx_ref = Arc::clone(&self.transaction);

        let shared_tx = {
            let mut lock = shared_tx_ref.lock().unwrap();
            lock.deref_mut().take()
        };

        if let Some(tx) = shared_tx {
            tx.commit().await?;
        }

        Ok(())
    }

    async fn transaction_rollback(&mut self) -> Result<()> {
        let shared_tx_ref = Arc::clone(&self.transaction);

        let shared_tx = {
            let mut lock = shared_tx_ref.lock().unwrap();
            lock.deref_mut().take()
        };

        if let Some(tx) = shared_tx {
            tx.rollback().await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Actions for PostgresPersistence<'_> {
    async fn create_account(&self) -> Result<()> {
        self.transaction_start().await?;

        sqlx::query!(
            "INSERT INTO accounts (name, author, isbn) VALUES ($1, $2, $3)",
            &book.title,
            &book.author,
            &book.isbn
        )
        .execute(&mut *self.transaction)
        .await
        .with_context(|| "Failed to create book")?;

        self.transaction_commit().await?;

        Ok(())
    }

    async fn read_account(&self) -> Result<()> {
        Ok(())
    }

    async fn update_account(&self) -> Result<()> {
        Ok(())
    }

    async fn delete_account(&self) -> Result<()> {
        Ok(())
    }
}
