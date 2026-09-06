use common::protocol::{ClientMessage, ServerMessage};
use server::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

/// Crea uno stato applicativo con un DB SQLite in memoria e avvia
/// `network::run_server` su una porta scelta dal SO,
/// restituendo l'indirizzo reale a cui connettersi.
pub async fn spawn_test_server() -> SocketAddr {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    server::db::init_schema(&db).await.unwrap();

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let (connections_notify, _) = tokio::sync::watch::channel(());
    let (chat_tx, _chat_rx) = tokio::sync::mpsc::unbounded_channel();
    let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();

    let state = AppState {
        db,
        connections: Arc::new(RwLock::new(HashMap::new())),
        user_status: Arc::new(RwLock::new(HashMap::new())),
        shutdown_tx,
        connections_notify,
        chat_tx,
        error_tx,
    };

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = server::network::run_server("127.0.0.1:0", state, ready_tx).await;
    });

    ready_rx.await.expect("il server di test deve avviarsi")
}

/// Apre una connessione TCP verso il server di test, restituendo lettore e
/// scrittore già pronti per `send`/`recv`.
pub async fn connect(addr: SocketAddr) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, writer) = stream.into_split();
    (BufReader::new(reader), writer)
}

/// Invia un `ClientMessage` codificato come farebbe il client vero (JSON + newline).
pub async fn send(writer: &mut OwnedWriteHalf, msg: &ClientMessage) {
    let mut payload = serde_json::to_string(msg).unwrap();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();
}

/// Legge una riga e la decodifica come `ServerMessage`.
pub async fn recv(reader: &mut BufReader<OwnedReadHalf>) -> ServerMessage {
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}
