mod draw;
mod input;
mod state;

use std::time::Duration;

use crate::ui::main_ui::state::UserChat;
use crate::ui::terminal_guard::TerminalGuard;
use crate::{state::IncomingChat, ui::main_ui::state::Users};
use common::protocol::ServerMessage;
use futures::StreamExt;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn run(
    state: &crate::state::AppState,
    mut chat_rx: UnboundedReceiver<IncomingChat>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(state).await;

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();

    // Sottoscrivi al canale di notifica delle connessioni
    let mut connections_rx = state.connections_notify.subscribe();

    loop {
        // Aggiorna i dati PRIMA di disegnare
        app.users = state
            .user_status
            .read()
            .await
            .iter()
            .clone()
            .map(|(username, status)| Users {
                username: username.clone(),
                status: status.status.clone(),
            })
            .collect();

        let user_connected: Vec<String> = state.connections.read().await.keys().cloned().collect();

        // Preserva la selezione: se l'utente selezionato è ancora connesso, mantieni l'indice
        let new_index = match app.connected_users.index_selected {
            Some(old_idx) => {
                let old_user = app.connected_users.connected_users.get(old_idx).cloned();
                old_user.and_then(|u| user_connected.iter().position(|c| c == &u.username))
            }
            None => None,
        };

        let old_chats = std::mem::take(&mut app.connected_users.connected_users);
        app.connected_users.connected_users = user_connected
            .into_iter()
            .map(|username| {
                // Cerca se esiste già una chat per questo utente e preservala
                if let Some(existing) = old_chats.iter().find(|uc| uc.username == username) {
                    existing.clone()
                } else {
                    UserChat {
                        username,
                        chat_log: Vec::new(),
                        chat_scroll: 0,
                    }
                }
            })
            .collect();

        app.connected_users.index_selected = new_index;

        terminal.draw(|frame| app.draw(frame))?;

        tokio::select! {
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_event))) if key_event.kind == KeyEventKind::Press  => {
                        match app.handle_key(key_event) {
                            input::Outbound::SendChat { message } => {
                                let timestamp = chrono::Utc::now();

                                // Prendi l'utente selezionato
                                if let Some(idx) = app.connected_users.index_selected {
                                    let selected_user = &app.connected_users.connected_users[idx];
                                    let username = &selected_user.username;

                                    // Invia SOLO al client selezionato
                                    let connections = state.connections.read().await;
                                    if let Some(tx) = connections.get(username) {
                                        let direct_msg = ServerMessage::DirectMessage {
                                            message: message.clone(),
                                            timestamp,
                                        };
                                        if let Err(e) = tx.send(direct_msg) {
                                            eprintln!("Errore durante l'invio del messaggio a {}: {}", username, e);
                                        }
                                    }

                                    // Salva il messaggio nella chat dell'utente selezionato
                                    app.connected_users.connected_users[idx].chat_log.push(
                                        state::ChatEntry {
                                            from_me: true,
                                            is_system: true,
                                            text: message,
                                            timestamp,
                                        },
                                    );
                                }
                            }
                            input::Outbound::SendBroadcast { message } => {
                                let timestamp = chrono::Utc::now();

                                let connections = state.connections.read().await;
                                for tx in connections.values() {
                                    let broadcast_msg = ServerMessage::BroadcastMessage {
                                        message: message.clone(),
                                        timestamp,
                                    };
                                    if let Err(e) = tx.send(broadcast_msg) {
                                        eprintln!("Errore durante l'invio del messaggio broadcast: {}", e);
                                    }
                                }

                                app.broadcast_log.push(state::BroadcastEntry { text: message, timestamp });
                            }

                            input::Outbound::Quit => {
                                let connections = state.connections.read().await;
                                for tx in connections.values() {
                                    let broadcast_msg = ServerMessage::BroadcastMessage {
                                            message: "Il server si sta arrestando, verrai disconnesso in 3 secondi...".to_string(),
                                            timestamp: chrono::Utc::now(),
                                        };
                                        if let Err(e) = tx.send(broadcast_msg) {
                                            eprintln!("Errore durante l'invio del messaggio broadcast: {}", e);
                                        }

                                        tokio::time::sleep(Duration::from_secs(3)).await;

                                        return Ok(())
                                }
                            },
                            input::Outbound::None => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(()),
                }
            }
            Some(incoming) = chat_rx.recv() => {
            // Trova la UserChat corrispondente al mittente
            if let Some(user_chat) = app.connected_users.connected_users
                .iter_mut()
                .find(|uc| uc.username == incoming.from_username)
            {
                user_chat.chat_log.push(state::ChatEntry {
                    from_me: false,
                    is_system: false,
                    text: incoming.message,
                    timestamp: incoming.timestamp,
                });
            }
        }

            // Quando le connessioni cambiano, il loop itera e ridisegna
            _ = connections_rx.changed() => {}
        }
    }
}
