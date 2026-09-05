mod draw;
mod input;
mod state;

use crate::movement_sim::MovementStatus;
use crate::ui::terminal_guard::TerminalGuard;
use common::protocol::{ClientMessage, ErrorContext, ServerMessage};
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use state::{BroadcastEntry, ChatEntry};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch; // Receiver

pub enum ExitReason {
    UserQuit,
    ConnectionLost,
    AccountDeleted,
}

pub async fn run(
    username: String,
    client_msg_tx: &Sender<ClientMessage>,
    server_msg_rx: &mut Receiver<ServerMessage>,
    movement_status_rx: &mut watch::Receiver<MovementStatus>,
    mut client_error_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
) -> Result<ExitReason, Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(username);

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();
    let mut movement_open = true;

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        tokio::select! {
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_event))) if key_event.kind == KeyEventKind::Press => {
                        match app.handle_key(key_event) {
                            Outbound::SendChat { message } => {
                                let timestamp = chrono::Utc::now();
                                if client_msg_tx.send(ClientMessage::ChatMessage { message: message.clone(), timestamp }).await.is_err() {
                                    return Ok(ExitReason::ConnectionLost);
                                }
                                app.chat_log.push(ChatEntry { from_me: true, is_system: false,text: message, timestamp });
                            }
                            Outbound::QueryStats { period } => {
                                if client_msg_tx.send(ClientMessage::QueryStats { period }).await.is_err() {
                                    return Ok(ExitReason::ConnectionLost);
                                }
                                app.stats_pending = true;
                            }
                            Outbound::DeleteAccount { password } => {
                                if client_msg_tx.send(ClientMessage::DeleteAccount { password }).await.is_err() {
                                    return Ok(ExitReason::ConnectionLost);
                                }
                            }
                            Outbound::Quit => return Ok(ExitReason::UserQuit),
                            Outbound::None => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(ExitReason::ConnectionLost),
                }
            }
            maybe_msg = server_msg_rx.recv() => {
                match maybe_msg {
                    Some(ServerMessage::DirectMessage { message, timestamp }) => {
                        app.chat_log.push(ChatEntry { from_me: false, is_system: false,text: message, timestamp });
                    }
                    Some(ServerMessage::BroadcastMessage{ message, timestamp }) => {
                        app.broadcast_log.push(BroadcastEntry { text: message, timestamp });
                    }
                    Some(ServerMessage::StatsResult { stats, timestamp }) => {
                        app.stats_pending = false;
                        app.stats = Some(stats);
                        app.stats_error = None;
                        app.stats_timestamp = Some(timestamp);
                    }
                    Some(ServerMessage::AccountDeleted { success: true, .. }) => {
                        return Ok(ExitReason::AccountDeleted);
                    }
                    Some(ServerMessage::AccountDeleted { success: false, reason, .. }) => {
                        app.delete_pending = false;
                        app.delete_step = state::DeleteAccountStep::EnterPassword;
                        app.delete_password.clear();
                        app.delete_error = Some(reason.unwrap_or_else(|| "Eliminazione dell'account fallita.".to_string()));
                    }
                    Some(ServerMessage::Error { message, context, timestamp }) => {
                        match context {
                            ErrorContext::Stats => {
                                app.stats_pending = false;
                                app.stats = None;
                                app.stats_error = Some(message);
                                app.stats_timestamp = Some(timestamp);
                            }
                            ErrorContext::Chat | ErrorContext::General => {
                                app.chat_log.push(ChatEntry {
                                    from_me: false,
                                    is_system: true,
                                    text: message,
                                    timestamp,
                                });
                            }
                        }
                    }
                    Some(_) => {} // StatsResult/altro
                    None => return Ok(ExitReason::ConnectionLost), // connessione persa
                }
            }
            result = movement_status_rx.changed(), if movement_open => {
                if result.is_err() {
                    movement_open = false;
                } else {
                    app.movement_status = movement_status_rx.borrow().clone();
                }
            }
            Some(message) = client_error_rx.recv() => {
                app.error_log.push(state::ErrorEntry {
                    text: message,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }
}
