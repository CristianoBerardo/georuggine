use crate::input::read_line;
use crate::state::AppState;
use common::protocol::ServerMessage;
use std::io::{self, Write};

/// Stampa il menu
pub fn print_menu() {
    println!("\n=== Menu Principale ===");
    println!("1. Invia messaggio broadcast");
    println!("2. Invia messaggio unicast");
    println!("3. Stampa stato degli utenti");
    print!("Scelta: ");
    let _ = io::stdout().flush();
}

pub async fn menu(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_menu();

    loop {
        let choice = read_line("")?;

        match choice.as_str() {
            // Invia messaggio broadcast
            "1" => {
                if state.connections.read().await.is_empty() {
                    eprintln!("Nessun utente connesso.");
                    print_menu();
                    continue;
                }

                let message = read_line("Messaggio: ")?;
                let connections = state.connections.read().await;
                for tx in connections.values() {
                    let broadcast_msg = ServerMessage::BroadcastMessage {
                        message: message.clone(),
                    };
                    if let Err(e) = tx.send(broadcast_msg) {
                        eprintln!("Errore durante l'invio del messaggio broadcast: {}", e);
                    }
                }
                print_menu();
            }
            // Invia messaggio unicast
            "2" => {
                let username = match choose_username(state).await {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("{}", e);
                        print_menu();
                        continue;
                    }
                };
                let connections = state.connections.read().await;
                loop {
                    match connections.get(&username) {
                        Some(tx) => {
                            let message = read_line("Messaggio: ")?;
                            let unicast_msg = ServerMessage::DirectMessage {
                                message: message.clone(),
                            };
                            if let Err(e) = tx.send(unicast_msg) {
                                eprintln!("Errore durante l'invio del messaggio unicast: {}", e);
                            }
                            break;
                        }
                        None => {
                            eprintln!("Username non trovato.");
                            continue;
                        }
                    }
                }
                print_menu();
            }
            // Stampa stato degli utenti
            "3" => {
                let user_status = state.user_status.read().await;
                println!("\n=== Stato degli utenti ===");
                for (username, info) in user_status.iter() {
                    println!("- {}: {:?}", username, info.state);
                }
                print_menu();
            }
            _ => {
                eprintln!("Scelta non valida. Riprova.");
                print_menu();
            }
        }
    }
}

async fn choose_username(
    state: &AppState,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let connections = state.connections.read().await;
    if connections.is_empty() {
        return Err("Nessun utente connesso".into());
    }
    println!("Utenti connessi:");
    for username in connections.keys() {
        println!("- {}", username);
    }
    let username = read_line("Username destinatario: ")?;
    Ok(username)
}
