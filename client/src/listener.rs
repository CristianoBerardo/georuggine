use common::protocol::ServerMessage;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;

// Ascolta in continuazione i messaggi asincroni del server
pub async fn listen(mut reader: BufReader<OwnedReadHalf>, server_msg_tx: Sender<ServerMessage>) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connessione chiusa dal server
                return;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ServerMessage>(trimmed) {
                    Ok(msg) => {
                        if server_msg_tx.send(msg).await.is_err() {
                            // Il ricevitore è stato chiuso, quindi usciamo dal loop
                            return;
                        }
                    }
                    Err(_) => {
                        // Ignora i messaggi malformati e continua ad ascoltare
                        // Non dovrebbe mai succedere in quanto client e server condividono lo
                        // stesso protocollo di messaggistica
                        continue;
                    }
                }
            }
            Err(_) => {
                return;
            }
        }
    }
}
