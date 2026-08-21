use common::protocol::ServerMessage;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

// Ascolta in continuazione i messaggi asincroni del server e li stampa a video
pub async fn listen(mut reader: BufReader<OwnedReadHalf>) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connessione chiusa dal server
                println!("\n[LISTENER] Connessione chiusa dal server.");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ServerMessage>(trimmed) {
                    Ok(msg) => match msg {
                        ServerMessage::BroadcastMessage { message } => {
                            println!("\n[LISTENER] Messaggio broadcast ricevuto: {}", message);
                        }
                        ServerMessage::DirectMessage { message } => {
                            println!("\n[LISTENER] Messaggio diretto ricevuto: {}", message);
                        }
                        ServerMessage::StatsResult { stats } => {
                            println!("\n[LISTENER] Statistiche ricevute: {:?}", stats);
                        }
                        ServerMessage::AuthResult { .. } => {
                            continue; // Ignora i messaggi di AuthResult, gestiti altrove
                        }
                        _ => {
                            println!("\n[LISTENER] Messaggio inatteso dal server: {:?}", msg);
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            "\n[LISTENER] Errore durante la deserializzazione del messaggio: {}",
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("\n[LISTENER] Errore durante la lettura: {}", e);
                break;
            }
        }
    }
}
