use alloy::primitives::{b256, Address, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{BlockId, BlockTransactionsKind, Filter, Log};
use tokio::time::{sleep, Duration}; 
use std::sync::Arc;
use std::cmp::{max, min};
use std::str::FromStr;
use crate::server::db::DbRepository;
use crate::models::models::{TransactionModel, Config};

const TRANSFER_EVENT_TOPIC: FixedBytes<32> = b256!("ddf252ad1e2e17e822157743b01e6a43b3b4f5144e1176b68b7320015b28de64");
const BATCH_SIZE: u64 = 50;

pub async fn run_indexer(
    db_repo: Arc<dyn DbRepository>,
    provider: Box<dyn Provider + Send + Sync>,
    config: Config,
) {
    let usdc_address = match Address::from_str(&config.usdc_contract_address) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Indexer: Invalid USDC_CONTRACT_ADDRESS: {}. Shutting down.", e);
            return;
        }
    };


    let db_next_block = match db_repo.get_last_saved_block().await {
        Ok(Some(last_block)) => (last_block + 1) as u64,
        Ok(None) => 0,
        Err(e) => {
            eprintln!("Indexer: Failed to get last block from DB: {}. Retrying in 10s.", e);
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
                eprintln!("Indexer: Failed to get block number: {}. Retrying in 10s.", e);
                sleep(Duration::from_secs(10)).await;
                return;
            }
        };
        println!("Indexer: Starting from latest network block {}", current_block_num);
    } else {
         println!("Indexer: Starting scan from block {}", current_block_num);
    }

    loop {
        let latest_block = match provider.get_block_number().await {
            Ok(num) => num,
            Err(e) => {
                eprintln!("Indexer: Failed to get latest block number: {}. Retrying...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        if current_block_num > latest_block {
            println!("Indexer: Caught up to head. Waiting for new blocks...");
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        let to_block = min(current_block_num + BATCH_SIZE - 1, latest_block);

        let filter = Filter::new()
            .address(usdc_address)
            .event_signature(TRANSFER_EVENT_TOPIC)
            .from_block(current_block_num)
            .to_block(to_block);

        println!("Indexer: Scanning block range {}-{}...", current_block_num, to_block);

        match provider.get_logs(&filter).await {
            Ok(logs) => {
                if !logs.is_empty() {
                    println!("Indexer: Found {} USDC events in range", logs.len());
                    
                    let block_time = match provider.get_block(BlockId::from(to_block), BlockTransactionsKind::Hashes).await {
                        Ok(Some(block)) => block.header.timestamp as i64,
                        _ => {
                           eprintln!("Indexer: Failed to get block header for {}. Retrying...", to_block);
                           sleep(Duration::from_secs(5)).await;
                           continue;
                        }
                    };

                    for log in logs {
                        if let Err(e) = process_log(log, block_time, db_repo.clone()).await {
                            eprintln!("Indexer: Failed to process log: {}", e);
                        }
                    }
                }
                
                current_block_num = to_block + 1;
            }
            Err(e) => {

                eprintln!("Indexer: Error fetching logs for range: {}. Retrying...", e);
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
    let value_data: &[u8] = &log.inner.data.data.0;
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
    Address::from_slice(&topic[12..]) 
}