mod draw;
mod input;
mod state;

use crate::ui::terminal_guard::TerminalGuard;
use common::protocol::{ClientMessage, ServerMessage};
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use state::ChatEntry;
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
                                if client_msg_tx.send(ClientMessage::ChatMessage { message: message.clone() }).await.is_err() {
                                    return Ok(());
                                }
                                app.chat_log.push(ChatEntry { from_me: true, text: message });
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
                    Some(ServerMessage::DirectMessage { message }) => {
                        app.chat_log.push(ChatEntry { from_me: false, text: message });
                    }
                    Some(ServerMessage::BroadcastMessage{ message}) => {
                        app.broadcast_log.push(message);
                    }
                    Some(_) => {} // StatsResult/altro
                    None => return Ok(()), // connessione persa
                }
            }
        }
    }
}
