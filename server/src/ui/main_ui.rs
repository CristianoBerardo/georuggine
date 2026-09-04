mod draw;
mod input;
mod state;

use crate::input::read_line;
#[allow(dead_code)]
use crate::ui::terminal_guard::TerminalGuard;
use common::protocol::{ClientMessage, ErrorContext, ServerMessage};
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::Event::{FocusGained, FocusLost};
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch; // Receiver

pub async fn run(
    // users: Vec<String>,
    // client_msg_tx: &Sender<ClientMessage>,
    // server_msg_rx: &mut Receiver<ServerMessage>,
    // movement_status_rx: &mut watch::Receiver<MovementStatus>,
    state: &crate::state::AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(state).await;

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();

    loop {
        terminal.draw(|frame| app.draw(frame))?;
        app.users = state.user_status.read().await.keys().cloned().collect();

        tokio::select! {
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_event))) if key_event.kind == KeyEventKind::Press  => {
                        match app.handle_key(key_event) {
                            input::Outbound::SendChat { message }  => {
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
                                app.chat_log.push(state::ChatEntry { from_me: true, is_system: false,text: message, timestamp });
                                    }
                                    input::Outbound::Quit => return Ok(()),
                                    input::Outbound::None => {}
                                }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(()),
                }
            }
        }
    }
}
