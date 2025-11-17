use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use crate::server::db::DbRepository;
use dotenv::dotenv;
use eyre::Result;
use std::env;
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct Config {
    pub mainnet_rpc_url: String,
    pub usdc_contract_address: String,
    pub start_block: Option<u64>,
    pub testnet_rpc_url: String,
    pub testnet_usdc_address: String,
    pub private_key: String,
    pub database_url: String, 
}
impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();
        let mainnet_rpc_url = env::var("MAINNET_RPC_URL")?;
        let usdc_contract_address = env::var("USDC_CONTRACT_ADDRESS")?;
        let testnet_rpc_url = env::var("TESTNET_RPC_URL")?;
        let testnet_usdc_address = env::var("TESTNET_USDC_ADDRESS")?;
        let private_key = env::var("PRIVATE_KEY")
            .map_err(|e| eyre::eyre!("PRIVATE_KEY not set: {}", e))?;
        let database_url = env::var("DATABASE_URL")?;
        let start_block = env::var("START_BLOCK").ok().and_then(|s| s.parse().ok());
        Ok(Config {
            mainnet_rpc_url,
            usdc_contract_address,
            start_block,
            testnet_rpc_url,
            testnet_usdc_address,
            private_key,
            database_url,
        })
    }
}
#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub to_address: String,
    pub amount_raw: String, 
}
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TransactionModel {
    pub tx_hash: String,
    pub log_index: i64,
    pub block_number: i64,
    pub sender: String,
    pub receiver: Option<String>,
    pub value_wei: String,
    pub tx_time: i64,
}
#[derive(Debug, Deserialize)]
pub struct TransactionFilters {
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub participant: Option<String>,
    pub start_time: Option<i64>, 
    pub end_time: Option<i64>,  
    pub page: Option<u32>, 
    pub page_size: Option<u32>
}
#[derive(Clone)]
pub struct AppState {
    pub db_repo: Arc<dyn DbRepository>,
    pub config: Arc<Config>, 
}