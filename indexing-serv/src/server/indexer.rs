
use alloy::providers::Provider; 
use alloy::rpc::types::{BlockId, BlockTransactions, BlockTransactionsKind};
use tokio::time::{sleep, Duration};
use std::sync::Arc;


use crate::server::db::DbRepository;
use crate::models::models::TransactionModel;


pub async fn run_indexer(
    db_repo: Arc<dyn DbRepository>,
    provider: Box<dyn Provider + Send + Sync>,
) {

    let last_saved_block_result = db_repo.get_last_saved_block().await;

    let mut current_block_num;
    match last_saved_block_result {
        Ok(last_block_opt) => {
            if let Some(last_block) = last_block_opt {
                current_block_num = (last_block + 1) as u64;
                println!("Indexer: Resuming from saved block {}", current_block_num);
            } else {
                println!("Indexer: Database is empty. Fetching latest block number...");
                current_block_num = match provider.get_block_number().await {
                    Ok(num) => num,
                    Err(e) => {
                        eprintln!("Indexer: Failed to get block number: {}. Retrying in 10s.", e);
                        sleep(Duration::from_secs(10)).await;
                        return;
                    }
                };
                println!("Indexer: Starting from latest block {}", current_block_num);
            }
        }
        Err(e) => {
            eprintln!("Indexer: Failed to get last block from DB: {}. Retrying in 10s.", e);
            sleep(Duration::from_secs(10)).await;
            return;
        }
    }

    println!("Indexer: Starting scan from block {}", current_block_num);

    loop {
        let block_id = BlockId::from(current_block_num);
        match provider.get_block(block_id, BlockTransactionsKind::Full).await {
            Ok(Some(block)) => {
                let block_time: i64 = block.header.timestamp as i64;
                println!("Indexer: Scanned block {} ({} txs)", current_block_num, block.transactions.len());

                if let BlockTransactions::Full(txs) = &block.transactions {
                    for tx in txs {
                        let tx_model = TransactionModel {
                            tx_hash: tx.hash.to_string(),
                            block_number: current_block_num as i64,
                            sender: tx.from.to_string(),
                            receiver: tx.to.map(|a| a.to_string()),
                            value_wei: tx.value.to_string(),
                            tx_time: block_time,
                        };

                        
                        if let Err(e) = db_repo.insert_transaction(&tx_model).await {
                            eprintln!("Indexer: Failed to insert tx {}: {}", tx_model.tx_hash, e);
                        }
                    }
                }
                current_block_num += 1;
            }
            Ok(None) => {
                println!("Indexer: Block {} not yet available. Waiting 5s...", current_block_num);
                sleep(Duration::from_secs(5)).await;
            }
            Err(e) => {
                eprintln!("Indexer: Error fetching block: {}. Retrying in 10s...", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}