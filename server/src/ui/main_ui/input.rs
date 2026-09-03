#[allow(dead_code)]
use super::state::App;
use common::protocol::TimePeriod;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    // SendChatMessage(String),
    // RequestStats(TimePeriod),
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        match key.code {
            KeyCode::Esc => return Outbound::Quit,
            _ => Outbound::None,
        }
    }
}
