use super::state::App;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        match key.code {
            KeyCode::Esc => Outbound::Quit,
            _ => Outbound::None,
        }
    }
}
