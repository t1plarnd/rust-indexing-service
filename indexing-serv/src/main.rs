// src/main.rs
use eyre::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod models;
mod server;

use models::models::{AppState, Config};
use server::api::run;
use server::db::{DbRepository, PgRepository};

#[tokio::main]
async fn main() -> Result<()> {
    let app_config = Config::load()?;
    println!("Config loaded.");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;
    println!("Database connected.");
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => println!("Migrations applied successfully."),
        Err(e) => {
            eprintln!("Migration error: {}", e);
            eprintln!("Try running: docker-compose down --volumes && docker-compose up");
            std::process::exit(1);
        }
    }

    let db_repo_impl = PgRepository::new(pool.clone());
    let db_repo: Arc<dyn DbRepository> = Arc::new(db_repo_impl);
    let app_state = AppState { 
        db_repo, 
        config: Arc::new(app_config.clone())
    };

    println!("Starting API server...");
    run(app_state, app_config).await?;

    Ok(())
}