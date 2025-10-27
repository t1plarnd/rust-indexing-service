

CREATE TABLE transactions (
    tx_hash TEXT PRIMARY KEY,
    block_number BIGINT NOT NULL,
    sender TEXT NOT NULL,
    receiver TEXT, 
    value_wei TEXT NOT NULL,
    tx_time TEXT NOT NULL
);


CREATE INDEX idx_transactions_sender ON transactions (sender);
