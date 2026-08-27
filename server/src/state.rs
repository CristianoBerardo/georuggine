use common::protocol::ServerMessage;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum UserState {
    Sconnesso,
    Fermo,
    InMovimento,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub state: UserState,
    pub s: u64,
}

pub type Username = String;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool, // Connessione al DB (contiene un Arc)
    pub connections: Arc<RwLock<HashMap<Username, UnboundedSender<ServerMessage>>>>,
    pub user_status: Arc<RwLock<HashMap<Username, Info>>>, // Stato degli utenti (connesso, fermo, in movimento)
}

// Connections:
// mappa degli utenti attualmente connessi al server: implementazione concreta della sessione TCP
// Hashmap: username, canale di comunicazione (Sender)
