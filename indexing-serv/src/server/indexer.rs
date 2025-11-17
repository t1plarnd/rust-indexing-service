use alloy::primitives::{b256, Address, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::cmp::{max, min};
use std::str::FromStr;
use crate::server::db::DbRepository;
use crate::models::models::{TransactionModel, Config};
use tracing::{info, error};

const TRANSFER_EVENT_TOPIC: FixedBytes<32> = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
const BATCH_SIZE: u64 = 100;

pub async fn run_indexer(
    db_repo: Arc<dyn DbRepository>,
    provider: impl Provider,
    config: Config,) {
    let usdc_address = match Address::from_str(&config.usdc_contract_address) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Indexer: Invalid MAINNET USDC_CONTRACT_ADDRESS: {}. Shutting down.", e);
            return;
        }
    };
    let db_next_block = match db_repo.get_last_saved_block().await {
        Ok(Some(last_block)) => (last_block + 1) as u64,
        Ok(None) => 0,
        Err(e) => {
            error!("Indexer: Failed to get last block from DB: {}. Retrying in 10s.", e);
            sleep(Duration::from_secs(10)).await;
            return;
        }
    };
    let config_start_block = config.start_block.unwrap_or(0);
    let mut current_block_num = max(db_next_block, config_start_block);
    if current_block_num == 0 {
        current_block_num = match provider.get_block_number().await {
            Ok(num) => num,
            Err(e) => {
                error!("Indexer: Failed to get block number from MAINNET: {}. Retrying in 10s.", e);
                sleep(Duration::from_secs(10)).await;
                return;
            }
        };
    }
    loop {
        let latest_block = match provider.get_block_number().await {
            Ok(num) => num,
            Err(e) => {
                error!("Indexer: Failed to get latest MAINNET block number: {}. Retrying...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        if current_block_num > latest_block {
            sleep(Duration::from_secs(10)).await;
            continue;
        }
        let to_block = min(current_block_num + BATCH_SIZE - 1, latest_block);
        let filter = Filter::new()
            .address(usdc_address)
            .event_signature(TRANSFER_EVENT_TOPIC)
            .from_block(current_block_num)
            .to_block(to_block);
        match provider.get_logs(&filter).await {
            Ok(logs) => {
                if !logs.is_empty() {                  
                    let block_time = match provider.get_block_by_number(to_block.into()).await {
                        Ok(Some(block)) => block.header.timestamp as i64,
                        Ok(None) => {
                            error!("Indexer: MAINNET Block {} not found. Retrying...", to_block);
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                        Err(e) => {
                            error!("Indexer: Failed to get MAINNET block header for {}: {}. Retrying...", to_block, e);
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    };
                    for log in logs {
                        if let Err(e) = process_log(log, block_time, db_repo.clone()).await {
                            error!("Indexer: Failed to process MAINNET log: {}", e);
                        }
                    }
                }
                current_block_num = to_block + 1;
            }
            Err(e) => {
                error!("Indexer: Error fetching MAINNET logs for range: {}. Retrying...", e);
                sleep(Duration::from_secs(5)).await;
                continue; 
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn process_log(log: Log, block_time: i64, db_repo: Arc<dyn DbRepository>) -> Result<(), String> {
    if log.topics().len() != 3 {
        return Err(format!("Invalid log topics length for tx: {}", log.transaction_hash.unwrap_or_default()));
    }
    let from = decode_address_from_topic(log.topics()[1]);
    let to = decode_address_from_topic(log.topics()[2]);
    let value_data: &[u8] = &log.data().data.0;
    let value = U256::from_be_slice(value_data);
    let tx_model = TransactionModel {
        tx_hash: log.transaction_hash.unwrap_or_default().to_string(),
        log_index: log.log_index.unwrap_or(0) as i64,
        block_number: log.block_number.unwrap_or(0) as i64,
        sender: from.to_string(),
        receiver: Some(to.to_string()),
        value_wei: value.to_string(),
        tx_time: block_time,
    };
    db_repo.insert_transaction(&tx_model).await
        .map_err(|e| e.to_string())
}

fn decode_address_from_topic(topic: FixedBytes<32>) -> Address {
    Address::from_slice(&topic[12..32])
}