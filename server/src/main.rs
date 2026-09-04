mod auth;
mod console;
mod db;
mod handlers;
mod input;
mod menu;
mod messaging;
mod network;
mod state;
mod stats;
mod ui;
mod user_status;

use crate::user_status::init_status_map;
use sqlx::sqlite::SqlitePool;
use state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connessione al database
    let pool = SqlitePool::connect("sqlite:data/georuggine.db").await?;
    println!("Pool fatto");

    // 2. Costruire lo stato condiviso
    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let (connections_notify, _) = tokio::sync::watch::channel(());
    let (chat_tx, chat_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState {
        db: pool,
        connections: Arc::new(RwLock::new(HashMap::new())),
        user_status: Arc::new(RwLock::new(HashMap::new())),
        shutdown_tx,
        connections_notify,
        chat_tx, // NUOVO
    };
    println!("Stato fatto");

    init_status_map(&mut state).await?;

    // 3. Avviare il server TCP in background, passandogli una copia dello stato
    let network_state = state.clone();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = network::run_server("127.0.0.1:8080", network_state, ready_tx).await {
            eprintln!("Errore nel server di rete: {}", e);
        }
    });

    // Aspetta che il server sia davvero in ascolto sulla porta prima di mostrare il menu
    let _ = ready_rx.await;

    ui::main_ui::run(&state, chat_rx).await?;

    // 4. Avviare il menu principale
    // menu::menu(&state).await?;

    Ok(())
}
