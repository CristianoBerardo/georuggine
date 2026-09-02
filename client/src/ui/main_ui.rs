mod draw;
mod input;
mod state;

use crate::ui::terminal_guard::TerminalGuard;
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};

pub async fn run(username: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(username);

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    // Crea uno stream di eventi dal terminale
    let mut term_events = EventStream::new();

    loop {
        terminal.draw(|frame| app.draw(frame))?;
        if let Some(Ok(Event::Key(key_event))) = term_events.next().await {
            if key_event.kind == KeyEventKind::Press {
                match app.handle_key(key_event) {
                    Outbound::Quit => return Ok(()),
                    Outbound::None => {}
                }
            }
        }
    }
}
