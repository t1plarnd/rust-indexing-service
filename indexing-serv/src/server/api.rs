use axum::{
    extract::{Path, State, Query},
    routing::{get, post},
    Json, Router,
};
use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use std::str::FromStr;
use alloy::primitives::{Address, U256};
use alloy::signers::local::PrivateKeySigner;
use crate::server::indexer::run_indexer;
use crate::models::models::{AppState, Config, TransactionFilters, TransactionModel, SendRequest};
use tracing::{info, error};

pub async fn run(app_state: AppState, app_config: Config) -> Result<()> {
    let repo_for_indexer = app_state.db_repo.clone();
    let config_clone = app_config.clone();
    tokio::spawn(async move {
        let provider = ProviderBuilder::new()
            .on_http(config_clone.mainnet_rpc_url.parse().unwrap());
        run_indexer(repo_for_indexer, provider, config_clone).await;
    });
    let app = Router::new()
        .route("/transactions/:hash", get(get_transaction_by_hash))
        .route("/transactions", get(get_transactions))
        .route("/send", post(send_transaction))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));   
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
async fn get_transaction_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionModel>, String> {
    match state.db_repo.get_transaction_by_hash(&hash).await {
        Ok(tx) => Ok(Json(tx)),
        Err(e) => Err(format!("Transaction not found: {}", e)),
    }
}
async fn get_transactions(
    State(state): State<AppState>,
    Query(filters): Query<TransactionFilters>,
) -> Result<Json<Vec<TransactionModel>>, String> {
    match state.db_repo.get_transactions(filters).await {
        Ok(txs) => Ok(Json(txs)),
        Err(e) => Err(format!("Failed to get transactions: {}", e)),
    }
}
#[axum::debug_handler]
async fn send_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<String>, String> {
    let to_addr = Address::from_str(&payload.to_address).map_err(|e| format!("Invalid 'to_address': {}", e))?;
    let amount = U256::from_str(&payload.amount_raw).map_err(|e| format!("Invalid 'amount_raw': {}", e))?;
    let _signer = PrivateKeySigner::from_str(&state.config.private_key).map_err(|e| format!("Invalid PRIVATE_KEY: {}", e))?;
    let _usdc_address = Address::from_str(&state.config.testnet_usdc_address).map_err(|e| format!("Invalid Testnet USDC Address: {}", e))?;
    Ok(Json(format!("Transaction prepared successfully!\n\n Details:\n• To: {}\n• Amount: {} USDC\n• Network: Sepolia Testnet\n• Status: Ready to send",
        to_addr, amount)))
}