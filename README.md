# Ethereum Indexer in Rust

This project indexes Ethereum transactions into a PostgreSQL database and provides a REST API to retrieve them.

The project uses `alloy-rs`, `axum`, `sqlx`, and `docker-compose`.

## How to Run

Uses Docker Compose to run the entire stack (API, Database, Nginx).

1.  **Requirements:**
    * Docker
    * Docker Compose

2.  **Clone the repository:**
    ```bash
    git clone https://github.com/t1plarnd/rust-ethereum-indexing-service.git
    cd your_project_name
    ```

3.  **Create the .env file:**
    Copy the env.example template and fill it with your data.
    ```bash
    cp .env.example .env
    ```
    You need to edit .env and insert your actual HTTP_INFURA_URL.

4.  **Start the service:**
    ```bash
    docker compose up --build
    ```

5.  **That's it!** The service is now available at http://localhost.

## API Endpoints

* GET /transactions - Get a list of recent transactions
* GET /transactions?sender=0x... - Filter by sender
* GET /transactions?receiver=0x... - Filter by receiver
* GET /transactions?participant=0x... - Filter by participant (sender OR receiver)
* GET /transactions/:hash - Get a single transaction by its hash
* POST /send - Prepare testnet transaction
