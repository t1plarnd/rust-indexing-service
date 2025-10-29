
CREATE TABLE IF NOT EXISTS transactions (
    tx_hash TEXT PRIMARY KEY,
    block_number BIGINT NOT NULL,
    sender TEXT NOT NULL,
    receiver TEXT,
    value_wei TEXT NOT NULL,
    tx_time BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions (sender);
CREATE INDEX IF NOT EXISTS idx_transactions_receiver ON transactions (receiver);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_time ON transactions (tx_time);