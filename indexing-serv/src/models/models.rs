use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use crate::server::db::DbRepository;
use dotenvy::dotenv; 
use eyre::Result;   
use std::env;      


#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub http_infura_url: String,
    pub start_block: Option<u64>,
    pub usdc_contract_address: String,
}
impl Config {

    pub fn load() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .map_err(|e| eyre::eyre!("DATABASE_URL not set: {}", e))?;
        let http_infura_url = env::var("HTTP_INFURA_URL")
            .map_err(|e| eyre::eyre!("HTTP_INFURA_URL not set: {}", e))?;
        let start_block = env::var("START_BLOCK").ok().and_then(|s| s.parse().ok());
        if start_block.is_some() {
            println!("Config: Found START_BLOCK={}", start_block.unwrap());
        }
        let usdc_contract_address = env::var("USDC_CONTRACT_ADDRESS")
            .map_err(|e| eyre::eyre!("USDC_CONTRACT_ADDRESS not set: {}", e))?;
        Ok(Config {
            database_url,
            http_infura_url,
            start_block,
            usdc_contract_address,
        })
    }
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
}