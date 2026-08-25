use crate::menu::print_menu;
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
                            println!("\n\n[LISTENER] Messaggio broadcast ricevuto:");
                            println!("  {}", message);
                            print_menu();
                        }
                        ServerMessage::DirectMessage { message } => {
                            println!("\n\n[LISTENER] Messaggio diretto ricevuto:");
                            println!("  {}", message);
                            print_menu();
                        }
                        ServerMessage::StatsResult { stats } => {
                            println!("\n\n[LISTENER] Statistiche ricevute:");
                            println!("  Distanza totale percorsa: {:.2} km", stats.distance_km);
                            println!("  Velocità media: {:.2} km/h", stats.avg_speed_kmh);
                            println!(
                                "  Tempo totale di movimento: {} h e {} min",
                                stats.moving_duration_secs / 3600,
                                (stats.moving_duration_secs % 3600) / 60
                            );
                            println!(
                                "  Tempo totale di inattività: {} h e {} min",
                                stats.paused_duration_secs / 3600,
                                (stats.paused_duration_secs % 3600) / 60
                            );
                            print_menu();
                        }
                        ServerMessage::AuthResult { .. } => {
                            continue; // Ignora i messaggi di AuthResult, gestiti in client/auth.rs
                        }
                        _ => {
                            println!("\n\n[LISTENER] Messaggio inatteso dal server:");
                            println!("  {:?}", msg);
                            print_menu();
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            "\n[LISTENER] Errore durante la deserializzazione del messaggio: {}",
                            e
                        );
                        print_menu();
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
