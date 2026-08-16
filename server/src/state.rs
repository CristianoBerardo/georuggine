use common::protocol::ServerMessage;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool, // Connessione al DB (contiene un Arc)
    pub connections: Arc<RwLock<HashMap<String, UnboundedSender<ServerMessage>>>>,
}

// Connections:
// mappa degli utenti attualmente connessi al server: implementazione concreta della sessione TCP
// Hashmap: username, canale di comunicazione (Sender)
