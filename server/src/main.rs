mod db;
mod network;
mod state;

use sqlx::sqlite::SqlitePool;
use state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connessione al database
    let pool = SqlitePool::connect("sqlite:data/georuggine.db").await?;
    println!("Pool fatto");

    // 2. Costruire lo stato condiviso
    let state = AppState {
        db: pool,
        connections: Arc::new(RwLock::new(HashMap::new())),
    };
    println!("Stato fatto");

    // 3. Avviare il server TCP passando lo stato a ogni connessione
    network::run_server("127.0.0.1:8080", state).await?;
    println!("Network fatto");

    Ok(())
}
