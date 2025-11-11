use crate::models::models::{TransactionFilters, TransactionModel};
use async_trait::async_trait;
use eyre::Result;
use sqlx::{PgPool, QueryBuilder, Error as SqlxError};


#[async_trait]
pub trait DbRepository: Send + Sync {
    async fn get_last_saved_block(&self) -> Result<Option<i64>, SqlxError>;
    async fn insert_transaction(&self, tx: &TransactionModel) -> Result<(), SqlxError>;
    async fn get_transaction_by_hash(&self, hash: &str) -> Result<TransactionModel, SqlxError>;
    async fn get_transactions(&self, filters: TransactionFilters) -> Result<Vec<TransactionModel>, SqlxError>;
}
#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}
impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DbRepository for PgRepository {
    async fn get_last_saved_block(&self) -> Result<Option<i64>, SqlxError> {
        match sqlx::query!("SELECT MAX(block_number) as max_num FROM transactions")
            .fetch_one(&self.pool)
            .await{
            Ok(record) => Ok(record.max_num),
            Err(SqlxError::RowNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn insert_transaction(&self, tx: &TransactionModel) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"INSERT INTO transactions (tx_hash, log_index, block_number, sender, receiver, value_wei, tx_time) 
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (tx_hash, log_index) DO NOTHING"#,
            tx.tx_hash,
            tx.log_index,
            tx.block_number,
            tx.sender,
            tx.receiver,
            tx.value_wei,
            tx.tx_time
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn get_transaction_by_hash(&self, hash: &str) -> Result<TransactionModel, SqlxError> {
        sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE tx_hash = $1",
            hash
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_transactions(&self, filters: TransactionFilters) -> Result<Vec<TransactionModel>, SqlxError> {
        let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new("SELECT * FROM transactions WHERE 1=1");
        let add_where = |builder: &mut QueryBuilder<sqlx::Postgres>| {builder.push(" AND ");};

        if let Some(sender) = filters.sender {
            add_where(&mut query_builder);
            query_builder.push("sender = ").push_bind(sender);
        }
        if let Some(receiver) = filters.receiver {
            add_where(&mut query_builder);
            query_builder.push("receiver = ").push_bind(receiver);
        }
        if let Some(participant) = filters.participant {
            add_where(&mut query_builder);
            query_builder.push("(sender = ").push_bind(participant.clone())
                         .push(" OR receiver = ").push_bind(participant).push(")");
        }
        if let Some(start_time) = filters.start_time {
            add_where(&mut query_builder);
            query_builder.push("tx_time >= ").push_bind(start_time);
        }
        if let Some(end_time) = filters.end_time {
            add_where(&mut query_builder);
            query_builder.push("tx_time <= ").push_bind(end_time);
        }

        query_builder.push(" ORDER BY block_number DESC, log_index DESC"); 
        let page_size = filters.page_size.unwrap_or(50).min(100); 
        let page = filters.page.unwrap_or(1);
        let offset = (page.saturating_sub(1)) * page_size;
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page_size as i64); 
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);
        query_builder.build_query_as::<TransactionModel>()
            .fetch_all(&self.pool)
            .await
    }
}