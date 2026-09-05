use common::protocol::ServerMessage;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct IncomingChat {
    pub from_username: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct IncomingError {
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UserStatus {
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
    pub connections_notify: watch::Sender<()>, // Notifica la TUI quando le connessioni cambiano
    pub chat_tx: mpsc::UnboundedSender<IncomingChat>, // Notifica la TUI quando arriva un messaggio chat da un client connesso
    pub error_tx: mpsc::UnboundedSender<IncomingError>, // Notifica la TUI di un errore avvenuto in un task non collegato alla UI
}

// Connections:
// mappa degli utenti attualmente connessi al server: implementazione concreta della sessione TCP
// Hashmap: username, canale di comunicazione (Sender)

// Shutdown_tx:
//ogni handle_connection si iscrive a questo canale e resta in ascolto; quando il menu del
// server invia un messaggio, ogni connessione attiva lo riceve, avvisa il proprio client e chiude la socket.
