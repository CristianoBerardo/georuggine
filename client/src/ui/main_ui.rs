mod draw;
mod input;
mod state;

use crate::ui::terminal_guard::TerminalGuard;
use common::protocol::{ClientMessage, ErrorContext, ServerMessage};
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use state::{BroadcastEntry, ChatEntry};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn run(
    username: String,
    client_msg_tx: &Sender<ClientMessage>,
    server_msg_rx: &mut Receiver<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(username);

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();
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
                                    return Ok(());
                                }
                                app.chat_log.push(ChatEntry { from_me: true, text: message, timestamp });
                            }
                            Outbound::QueryStats { period } => {
                                if client_msg_tx.send(ClientMessage::QueryStats { period }).await.is_err() {
                                    return Ok(());
                                }
                                app.stats_pending = true;
                            }
                            Outbound::Quit => return Ok(()),
                            Outbound::None => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(()),
                }
            }
            maybe_msg = server_msg_rx.recv() => {
                match maybe_msg {
                    Some(ServerMessage::DirectMessage { message, timestamp }) => {
                        app.chat_log.push(ChatEntry { from_me: false, text: message, timestamp });
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
                    Some(ServerMessage::Error { message, context, timestamp }) => {
                        match context {
                            ErrorContext::Stats => {
                                app.stats_pending = false;
                                app.stats = None;
                                app.stats_error = Some(message);
                                app.stats_timestamp = Some(timestamp);
                            }
                            ErrorContext::Chat | ErrorContext::General => {
                                // Nessun pannello dedicato per ora: ignorato.
                            }
                        }
                    }
                    Some(_) => {} // StatsResult/altro
                    None => return Ok(()), // connessione persa
                }
            }
        }
    }
}
