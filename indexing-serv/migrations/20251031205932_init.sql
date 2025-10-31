CREATE TABLE IF NOT EXISTS transactions (
    tx_hash VARCHAR(66) NOT NULL,
    log_index BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    sender VARCHAR(42) NOT NULL,
    receiver VARCHAR(42),
    value_wei TEXT NOT NULL,
    tx_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (tx_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_transactions_block_number ON transactions(block_number DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions(sender);
CREATE INDEX IF NOT EXISTS idx_transactions_receiver ON transactions(receiver);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_time ON transactions(tx_time);
