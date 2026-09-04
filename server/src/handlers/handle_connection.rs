use crate::db::get_user_by_username;
use crate::handlers::{
    handle_login::handle_login, handle_new_track_point::handle_new_track_point,
    handle_registration::handle_registration, handle_stats::handle_stats,
};
use crate::menu;
use crate::messaging::{receive_message, send_message};
use crate::state::AppState;
use crate::state::UserStatus;
use crate::user_status::{update_user_seconds, update_user_status};

use common::protocol::{ClientMessage, ErrorContext, ServerMessage};
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Instant;

const WATCHDOG_TIMEOUT_SECS: u64 = 35;

async fn clean_connection(
    state: &AppState,
    authenticated_user: &mut Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(user) = authenticated_user {
        let mut conns = state.connections.write().await;
        conns.remove(user.as_str());
        drop(conns);
        let _ = state.connections_notify.send(());
        update_user_status(state, user, UserStatus::Sconnesso).await?;
        update_user_seconds(state, user, 0).await?;
        println!("Utente {} disconnesso.", user);
    }
    Ok(())
}

pub async fn handle_connection(
    socket: TcpStream,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut authenticated_user: Option<String> = None;
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    let mut shutdown_rx = state.shutdown_tx.subscribe();
    let mut watchdog_deadline: Option<Instant> = None; // nessun timer attivo

    loop {
        tokio::select! {
            // Segnale di arresto del server (menu "Autodistruzione"): avvisa
            // il client e chiude la connessione, così viene raggiunto anche
            // il codice di pulizia subito sotto al loop.
            _ = shutdown_rx.recv() => {
                let notice = ServerMessage::BroadcastMessage {
                    message: "Il server si sta arrestando, verrai disconnesso.".to_string(),
                    timestamp: chrono::Utc::now(),
                };
                let _ = send_message(&mut writer, &notice).await;

                // Pulizia connessione quando il client si disconnette
                clean_connection(&state, &mut authenticated_user).await?;
                break;
            }
            // Ricezione messaggi dal client
            result = receive_message(&mut reader) => {
                let client_msg = match result {
                    Ok(None) => {

                        // Pulizia connessione quando il client si disconnette
                        clean_connection(&state, &mut authenticated_user).await?;
                        // menu::print_menu();

                        break; // Connessione chiusa dal client
                    }
                    Ok(Some(msg)) => msg,
                    Err(e) => {
                        let err_msg = ServerMessage::Error {
                            message: format!("Formato messaggio non valido: {}", e),
                            timestamp: chrono::Utc::now(),
                            context: ErrorContext::General,
                        };
                        send_message(&mut writer, &err_msg).await?;
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::Login { username, password } => {
                        handle_login(
                            username,
                            password,
                            &state,
                            &mut writer,
                            &tx,
                            &mut authenticated_user,
                        ).await?;
                    }
                    ClientMessage::Register { username, password } => {
                        handle_registration(
                            username,
                            password,
                            &state,
                            &mut writer,
                            &tx,
                            &mut authenticated_user,
                        ).await?;
                    }
                    ClientMessage::PositionUpdate { position } => {
                        if let Some(username) = &authenticated_user {
                            //println!("Aggiornamento posizione da {}: {:?}", username, position);
                            match get_user_by_username(&state.db, username).await? {
                                Some(user) => {
                                    handle_new_track_point(&state, &user, position).await?;
                                    watchdog_deadline = Some(Instant::now() + Duration::from_secs(WATCHDOG_TIMEOUT_SECS));

                                }
                                None => {
                                    let err_msg = ServerMessage::Error {
                                        message: "Utente non trovato.".to_string(),
                                        timestamp: chrono::Utc::now(),
                                        context: ErrorContext::General,
                                    };
                                    send_message(&mut writer, &err_msg).await?;
                                }
                            }
                        } else {
                            let err_msg = ServerMessage::Error {
                                message: "Devi essere autenticato per inviare aggiornamenti di posizione.".to_string(),
                                timestamp: chrono::Utc::now(),
                                context: ErrorContext::General,
                            };
                            send_message(&mut writer, &err_msg).await?;
                        }
                    }
                    ClientMessage::ChatMessage { message, timestamp } => {
                        if let Some(username) = &authenticated_user {
                            let incoming = crate::state::IncomingChat {
                                from_username: username.clone(),
                                message,
                                timestamp,
                            };
                            let _ = state.chat_tx.send(incoming);
                        }
                    }
                    // ClientMessage::ChatMessage { message, timestamp } => {
                    //     let text = format!(
                    //         "\n\n[{}] Messaggio ricevuto da {}: {}",
                    //         timestamp.with_timezone(&chrono::Local).format("%H:%M:%S"),
                    //         authenticated_user.as_ref().unwrap(),
                    //         message
                    //     );

                    //     // menu::print_or_queue_with_menu(text);
                    // }
                    ClientMessage::QueryStats { period } => {
                        handle_stats(
                            &state,
                            &mut writer,
                            authenticated_user.as_ref().unwrap(),
                            period,
                        ).await?;
                    }
                }
            }
            // Invio messaggi accodati nel canale mpsc verso il client
            Some(outgoing_msg) = rx.recv() => {
                send_message(&mut writer, &outgoing_msg).await?;
            }
            () = tokio::time::sleep_until(watchdog_deadline.unwrap_or_else(Instant::now)), if watchdog_deadline.is_some() => {
                if let Some(user) = &authenticated_user {
                    update_user_status(&state, user, UserStatus::Problema).await?;
                }
                watchdog_deadline = None;
            }
        }
    }

    Ok(())
}
