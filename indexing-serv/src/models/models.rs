use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use crate::server::db::DbRepository;
use dotenvy::dotenv; // <-- Додайте use
use eyre::Result;    // <-- Додайте use
use std::env;       // <-- Додайте use

/// Конфігурація додатка, завантажена з .env
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub http_infura_url: String,
}

// --- ОСНОВНА ЛОГІКА ТЕПЕР ТУТ ---
impl Config {
    /// Завантажує конфігурацію з .env файлу
    pub fn load() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .map_err(|e| eyre::eyre!("DATABASE_URL not set: {}", e))?;
        let http_infura_url = env::var("HTTP_INFURA_URL")
            .map_err(|e| eyre::eyre!("HTTP_INFURA_URL not set: {}", e))?;
        Ok(Config {
            database_url,
            http_infura_url,
        })
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TransactionModel {
    pub tx_hash: String,
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
}


#[derive(Clone)]
pub struct AppState {
    pub db_repo: Arc<dyn DbRepository>,
}