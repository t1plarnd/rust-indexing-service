use axum::{
    extract::{Path, State, Query},
    routing::get,
    Json, Router,
};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{BlockId, BlockTransactions, BlockTransactionsKind};
use eyre::Result;
use sqlx::{postgres::PgPoolOptions, PgPool, FromRow};
use sqlx::QueryBuilder;
use std::time::Duration;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use dotenvy::dotenv;
use std::env;


#[derive(Debug, Serialize, FromRow)]
struct TransactionModel {
    tx_hash: String,
    block_number: i64,
    sender: String,
    receiver: Option<String>,
    value_wei: String,
    tx_time: String,
}

#[derive(Debug, Deserialize)]
struct TransactionFilters {
    sender: Option<String>,
    receiver: Option<String>,
    participant: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {

    dotenv().ok();
    

    let db_connection_str = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_str)
        .await?;
    println!("Database connected.");


    sqlx::migrate!().run(&pool).await?;

    let rpc_url = env::var("HTTP_INFURA_URL")
        .expect("HTTP_INFURA_URL must be set");
    let provider = ProviderBuilder::new()
        .on_http(rpc_url.parse()?)
        .boxed(); 


    let app_state = AppState { db_pool: pool.clone() };

    tokio::spawn(run_indexer(pool, provider));

    let app = Router::new()
        .route("/transactions/:hash", get(get_transaction_by_hash))
        .route("/transactions", get(get_transactions))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any));


    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


// BACKGROUND INDEXER


async fn run_indexer(pool: PgPool, provider: impl Provider) {

    let last_saved_block: Option<i64> = sqlx::query!(
        "SELECT MAX(block_number) as max_num FROM transactions"
    )
    .fetch_one(&pool)
    .await
    .map(|rec| rec.max_num) 
    .unwrap_or(None); 

    let mut current_block_num;
    if let Some(last_block) = last_saved_block {
        current_block_num = (last_block + 1) as u64;
        println!("Indexer: Resuming from saved block {}", current_block_num);
    } else {
        current_block_num = provider.get_block_number().await.unwrap_or(0);
        println!("Indexer: Database is empty. Starting from latest block {}", current_block_num);
    }
    println!("Indexer: Starting scan from block {}", current_block_num);

    loop {
        let block_id = BlockId::from(current_block_num);
        match provider.get_block(block_id, BlockTransactionsKind::Full).await {
            Ok(Some(block)) => {
                let block_time_str = block.header.timestamp.to_string();

                println!("Indexer: Scanned block {} ({} txs)", current_block_num, block.transactions.len());
                if let BlockTransactions::Full(txs) = &block.transactions {
                    for tx in txs {
                        sqlx::query!(
                            r#"INSERT INTO transactions (tx_hash, block_number, sender, receiver, value_wei, tx_time) 
                               VALUES ($1, $2, $3, $4, $5, $6)
                               ON CONFLICT (tx_hash) DO NOTHING"#,
                            tx.hash.to_string(),
                            current_block_num as i64,
                            tx.from.to_string(),
                            tx.to.map(|a| a.to_string()),
                            tx.value.to_string(),
                            block_time_str 
                        )
                        .execute(&pool)
                        .await
                        .ok(); 
                    }
                }
                
                current_block_num += 1;
            }
            Ok(None) => {
                println!("Indexer: Block {} not yet available. Waiting 5s...", current_block_num);
                tokio::time::sleep(Duration::from_secs(5)).await;
            },
            Err(e) => {
                eprintln!("Indexer: Error fetching block: {}. Retrying...", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }
}


// API HANDLERS (No changes needed here)


async fn get_transaction_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionModel>, String> {
    
    let tx = sqlx::query_as!(
        TransactionModel,
        "SELECT * FROM transactions WHERE tx_hash = $1",
        hash
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(tx))
}


async fn get_transactions(
    State(state): State<AppState>,
    Query(filters): Query<TransactionFilters>, 
) -> Result<Json<Vec<TransactionModel>>, String> {
    
    let mut query_builder: QueryBuilder<sqlx::Postgres> = 
        QueryBuilder::new("SELECT * FROM transactions");

    let mut needs_and = false;
    let mut add_where = |builder: &mut QueryBuilder<sqlx::Postgres>, needs_and: &mut bool| {
        if *needs_and {
            builder.push(" AND ");
        } else {
            builder.push(" WHERE ");
            *needs_and = true;
        }
    };

    if let Some(sender) = filters.sender {
        add_where(&mut query_builder, &mut needs_and);
        query_builder.push("sender = ");
        query_builder.push_bind(sender);
    }

    if let Some(receiver) = filters.receiver {
        add_where(&mut query_builder, &mut needs_and);
        query_builder.push("receiver = ");
        query_builder.push_bind(receiver);
    }

    if let Some(participant) = filters.participant {
        add_where(&mut query_builder, &mut needs_and);
        query_builder.push("(sender = ");
        query_builder.push_bind(participant.clone());
        query_builder.push(" OR receiver = ");
        query_builder.push_bind(participant);
        query_builder.push(")");
    }

)
    if let Some(start_time) = filters.start_time {
        add_where(&mut query_builder, &mut needs_and);
        query_builder.push("tx_time >= ");
        query_builder.push_bind(start_time);
    }

    if let Some(end_time) = filters.end_time {
        add_where(&mut query_builder, &mut needs_and);
        query_builder.push("tx_time <= ");
        query_builder.push_bind(end_time);
    }

    query_builder.push(" ORDER BY block_number DESC LIMIT 50");

    let query = query_builder.build_query_as::<TransactionModel>();
    
    let txs = query.fetch_all(&state.db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(txs))
}