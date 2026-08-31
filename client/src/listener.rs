use crate::menu;
use common::protocol::ServerMessage;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;

// Ascolta in continuazione i messaggi asincroni del server e li stampa a video
// o li accoda se l'utente sta scrivendo.
// Inoltra i messaggi di autenticazione al task di login/registrazione.
pub async fn listen(
    mut reader: BufReader<OwnedReadHalf>,
    auth_resp_tx: Sender<ServerMessage>,
    menu_active: Arc<AtomicBool>,
) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connessione chiusa dal server
                println!("\n[LISTENER] Connessione chiusa dal server.");
                std::process::exit(0);
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ServerMessage>(trimmed) {
                    Ok(msg) => match msg {
                        ServerMessage::BroadcastMessage { message } => {
                            menu::print_or_queue_with_menu(
                                format!(
                                    "\n\n[LISTENER] Messaggio broadcast ricevuto:\n  {}",
                                    message
                                ),
                                &menu_active,
                            );
                        }
                        ServerMessage::DirectMessage { message } => {
                            menu::print_or_queue_with_menu(
                                format!(
                                    "\n\n[LISTENER] Messaggio diretto ricevuto:\n  {}",
                                    message
                                ),
                                &menu_active,
                            );
                        }
                        ServerMessage::StatsResult { stats } => {
                            // Chiude la protezione avviata dal branch "2" del
                            // menu (vedi `menu::end_wait_with`): le
                            // statistiche vengono mostrate per prime, prima
                            // di eventuali messaggi accodati nel frattempo.
                            menu::end_wait_with(
                                format!(
                                    "\n\n[LISTENER] Statistiche ricevute:\n  Distanza totale percorsa: {:.2} km\n  Velocità media: {:.2} km/h\n  Tempo totale di movimento: {} h e {} min\n  Tempo totale di inattività: {} h e {} min",
                                    stats.distance_km,
                                    stats.avg_speed_kmh,
                                    stats.moving_duration_secs / 3600,
                                    (stats.moving_duration_secs % 3600) / 60,
                                    stats.paused_duration_secs / 3600,
                                    (stats.paused_duration_secs % 3600) / 60
                                ),
                                &menu_active,
                            );
                        }
                        ServerMessage::AuthResult { .. } => {
                            // Inoltra l'esito di autenticazione al flusso di auth
                            let _ = auth_resp_tx.send(msg).await;
                        }
                        ServerMessage::Error { ref message } => {
                            eprintln!("\n[LISTENER] Errore dal server: {}", message);
                            let _ = auth_resp_tx.send(msg).await;
                        }
                    },
                    Err(e) => {
                        menu::print_or_queue_with_menu(
                            format!(
                                "\n[LISTENER] Errore durante la deserializzazione del messaggio: {}",
                                e
                            ),
                            &menu_active,
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("\n[LISTENER] Errore durante la lettura: {}", e);
                std::process::exit(0);
            }
        }
    }
}
