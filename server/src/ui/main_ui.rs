mod draw;
mod input;
mod state;

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
    username: String,
    // client_msg_tx: &Sender<ClientMessage>,
    // server_msg_rx: &mut Receiver<ServerMessage>,
    // movement_status_rx: &mut watch::Receiver<MovementStatus>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(username);

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();

    loop {
        terminal.draw(|frame| app.draw_placeholder(frame, frame.size(), "Test", false))?;

        tokio::select! {
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_event))) if key_event.kind == KeyEventKind::Press  => {
                        match app.handle_key(key_event) {
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
