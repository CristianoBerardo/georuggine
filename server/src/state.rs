use common::protocol::ServerMessage;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Sconnesso,
    Fermo,
    InMovimento,
    Problema,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub status: UserStatus,
    pub s: u64,
}

pub type Username = String;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool, // Connessione al DB (contiene un Arc)
    pub connections: Arc<RwLock<HashMap<Username, UnboundedSender<ServerMessage>>>>,
    pub user_status: Arc<RwLock<HashMap<Username, Info>>>, // Stato degli utenti (connesso, fermo, in movimento)
    pub shutdown_tx: broadcast::Sender<()>,
}

// Connections:
// mappa degli utenti attualmente connessi al server: implementazione concreta della sessione TCP
// Hashmap: username, canale di comunicazione (Sender)

// Shutdown_tx:
//ogni handle_connection si iscrive a questo canale e resta in ascolto; quando il menu del
// server invia un messaggio, ogni connessione attiva lo riceve, avvisa il proprio client e chiude la socket.
