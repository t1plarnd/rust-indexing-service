
use eyre::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;


use indexing_svc::models::models::{AppState, Config}; 
use indexing_svc::server::api::run;
use indexing_svc::server::db::{DbRepository, PgRepository};


#[tokio::main]
async fn main() -> Result<()> {

    let app_config = Config::load()?;
    println!("Config loaded.");


    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;
    println!("Database connected.");
    sqlx::migrate!().run(&pool).await?;
    println!("Migrations applied.");
    let db_repo_impl = PgRepository::new(pool.clone());
    let db_repo: Arc<dyn DbRepository> = Arc::new(db_repo_impl);
    let app_state = AppState { db_repo };

    println!("Starting API server and indexer...");
    run(app_state, app_config).await?;

    Ok(())
}