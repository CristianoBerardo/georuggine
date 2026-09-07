#![allow(dead_code)]

// ============================================================================
// Modulo di supporto condiviso per TUTTI i test del client:
//
// 1. Helper per test di integrazione con MOCK server (spawn_mock_server, ecc.)
// 2. Helper per test E2E con SERVER REALE (spawn_test_server, HeadlessClient)
// ============================================================================

use common::models::Position;
use common::protocol::{ClientMessage, ServerMessage};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;

// ============================================================================
// PARTE 1: Helper per test di integrazione con MOCK server
// ============================================================================

/// Crea una coppia TCP locale: restituisce (lato_server, lato_client) già connessi.
pub async fn mock_connection() -> (
    (BufReader<OwnedReadHalf>, OwnedWriteHalf), // Socket lato Server
    (BufReader<OwnedReadHalf>, OwnedWriteHalf), // Socket lato Client
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client_task = tokio::spawn(async move {
        connect(addr).await
    });
    let (server_stream, _) = listener.accept().await.unwrap();
    let (server_reader, server_writer) = server_stream.into_split();
    let (client_reader, client_writer) = client_task.await.unwrap();
    (
        (BufReader::new(server_reader), server_writer),
        (client_reader, client_writer),
    )
}

/// Apre una connessione TCP verso il server (mock o reale).
pub async fn connect(addr: SocketAddr) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, writer) = stream.into_split();
    (BufReader::new(reader), writer)
}

/// Invia un `ServerMessage` codificato come JSON + newline (usato lato mock server).
pub async fn send_server_msg(writer: &mut OwnedWriteHalf, msg: &ServerMessage) {
    let mut payload = serde_json::to_string(msg).unwrap();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}

/// Legge una riga dal socket e la decodifica come `ServerMessage`.
pub async fn recv(reader: &mut BufReader<OwnedReadHalf>) -> ServerMessage {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

/// Legge una riga dal socket e la decodifica come `ClientMessage`
/// (usato lato mock server per leggere ciò che il client ha inviato).
pub async fn recv_client_msg(reader: &mut BufReader<OwnedReadHalf>) -> ClientMessage {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

/// Scrive una stringa raw + newline sul socket, senza validazione JSON.
pub async fn send_raw(writer: &mut OwnedWriteHalf, raw: &str) {
    let mut payload = raw.to_string();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}

// ============================================================================
// PARTE 2: Helper per test E2E con SERVER REALE
// ============================================================================

/// Crea uno stato applicativo con un DB SQLite in memoria e avvia
/// `server::network::run_server` su una porta scelta dal SO.
/// Ricalca `server/tests/support/mod.rs` ma vive nel crate client.
pub async fn spawn_test_server() -> (
    SocketAddr,
    server::state::AppState,
    mpsc::UnboundedReceiver<server::state::IncomingChat>,
) {
    use sqlx::sqlite::SqlitePoolOptions;

    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    server::db::init_schema(&db).await.unwrap();

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let (connections_notify, _) = tokio::sync::watch::channel(());
    let (chat_tx, chat_rx) = mpsc::unbounded_channel();
    let (error_tx, _error_rx) = mpsc::unbounded_channel();

    let state = server::state::AppState {
        db,
        connections: Arc::new(RwLock::new(HashMap::new())),
        user_status: Arc::new(RwLock::new(HashMap::new())),
        shutdown_tx,
        connections_notify,
        chat_tx,
        error_tx,
    };

    let state_for_server = state.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = server::network::run_server("127.0.0.1:0", state_for_server, ready_tx).await;
    });

    let addr = ready_rx.await.expect("il server di test deve avviarsi");
    (addr, state, chat_rx)
}

/// Client headless: replica la topologia di `main.rs` (TCP → listener task →
/// writer task → canali MPSC) senza alcuna dipendenza dalla TUI.
pub struct HeadlessClient {
    pub client_msg_tx: mpsc::Sender<ClientMessage>,
    pub server_msg_rx: mpsc::Receiver<ServerMessage>,
    listener_handle: JoinHandle<()>,
    writer_handle: JoinHandle<()>,
}

impl HeadlessClient {
    /// Connette al server, avvia listener e writer task come `main.rs`.
    pub async fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let reader = BufReader::new(reader);

        let (client_msg_tx, mut client_msg_rx) = mpsc::channel::<ClientMessage>(100);
        let (server_msg_tx, server_msg_rx) = mpsc::channel::<ServerMessage>(100);

        let listener_handle = tokio::spawn(client::listener::listen(reader, server_msg_tx));

        let writer_handle = tokio::spawn(async move {
            while let Some(msg) = client_msg_rx.recv().await {
                if client::messaging::send_message(&mut writer, &msg)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            client_msg_tx,
            server_msg_rx,
            listener_handle,
            writer_handle,
        }
    }

    /// Invia Register e attende AuthResult.
    pub async fn register(&mut self, username: &str, password: &str) -> ServerMessage {
        self.client_msg_tx
            .send(ClientMessage::Register {
                username: username.to_string(),
                password: password.to_string(),
            })
            .await
            .unwrap();
        self.server_msg_rx.recv().await.expect("atteso AuthResult")
    }

    /// Invia Login e attende AuthResult.
    /// NOTA: dopo un login con successo, il server invia anche un DirectMessage
    /// di benvenuto che resta nel canale e va letto con `recv()`.
    pub async fn login(&mut self, username: &str, password: &str) -> ServerMessage {
        self.client_msg_tx
            .send(ClientMessage::Login {
                username: username.to_string(),
                password: password.to_string(),
            })
            .await
            .unwrap();
        self.server_msg_rx.recv().await.expect("atteso AuthResult")
    }

    /// Invia un ChatMessage. Il server non risponde via TCP.
    pub async fn send_chat(&mut self, message: &str) {
        self.client_msg_tx
            .send(ClientMessage::ChatMessage {
                message: message.to_string(),
                timestamp: chrono::Utc::now(),
            })
            .await
            .unwrap();
    }

    /// Invia un PositionUpdate. Il server non risponde via TCP ma salva nel DB.
    pub async fn send_position(&mut self, lat: f64, lon: f64) {
        self.client_msg_tx
            .send(ClientMessage::PositionUpdate {
                position: Position {
                    lat,
                    lon,
                    timestamp: chrono::Utc::now(),
                },
            })
            .await
            .unwrap();
    }

    /// Invia DeleteAccount e attende AccountDeleted.
    pub async fn delete_account(&mut self, password: &str) -> ServerMessage {
        self.client_msg_tx
            .send(ClientMessage::DeleteAccount {
                password: password.to_string(),
            })
            .await
            .unwrap();
        self.server_msg_rx
            .recv()
            .await
            .expect("atteso AccountDeleted")
    }

    /// Riceve il prossimo ServerMessage dal canale.
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        self.server_msg_rx.recv().await
    }

    /// Riceve con timeout. Restituisce `None` se scade senza messaggi.
    pub async fn try_recv_timeout(&mut self, duration: Duration) -> Option<ServerMessage> {
        tokio::time::timeout(duration, self.server_msg_rx.recv())
            .await
            .ok()
            .flatten()
    }

    /// Chiude la connessione in modo pulito: il writer drena i messaggi
    /// rimanenti, poi il socket viene chiuso, e il server rileva la disconnessione.
    pub async fn disconnect(self) {
        drop(self.client_msg_tx); // Chiude il sender → il writer esce dal loop
        let _ = self.writer_handle.await; // Attende che il writer finisca
        self.listener_handle.abort(); // Ferma il listener
    }
}
