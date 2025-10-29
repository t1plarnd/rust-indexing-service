// src/server/api.rs
use axum::{
    extract::{Path, State, Query},
    routing::get,
    Json, Router,
};
use alloy::providers::{Provider, ProviderBuilder}; // Виправлено
use eyre::Result;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

// Імпортуємо наші моделі, трейт та індексатор
use crate::models::models::{AppState, Config, TransactionFilters, TransactionModel};
use crate::server::indexer::run_indexer;

pub async fn run(app_state: AppState, app_config: Config) -> Result<()> {
    let rpc_url = app_config.http_infura_url;

    // --- ОСЬ ВИПРАВЛЕННЯ ---
    // Ми маємо завершити "конструктор" провайдера
    let provider: Box<dyn Provider + Send + Sync> = Box::new(ProviderBuilder::new()
        .on_http(rpc_url.parse()?) // Вказуємо URL
        .boxed()); // "Пакуємо" його в Box<dyn Trait>

    // Передаємо індексатору `Arc<dyn DbRepository>`
    let repo_for_indexer = app_state.db_repo.clone();
    tokio::spawn(run_indexer(repo_for_indexer, provider));

    // Створюємо роутер
    let app = Router::new()
        .route("/transactions/:hash", get(get_transaction_by_hash))
        .route("/transactions", get(get_transactions))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any));

    // Запускаємо сервер
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- Обробники API (без змін) ---
async fn get_transaction_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionModel>, String> {
    match state.db_repo.get_transaction_by_hash(&hash).await {
        Ok(tx) => Ok(Json(tx)),
        Err(e) => Err(e.to_string()),
    }
}

async fn get_transactions(
    State(state): State<AppState>,
    Query(filters): Query<TransactionFilters>,
) -> Result<Json<Vec<TransactionModel>>, String> {
    match state.db_repo.get_transactions(filters).await {
        Ok(txs) => Ok(Json(txs)),
        Err(e) => Err(e.to_string()),
    }
}