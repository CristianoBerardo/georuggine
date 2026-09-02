use super::state::{App, Panel};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panel::UserInfo => Panel::Movement,
                    Panel::Movement => Panel::StatsPeriod,
                    Panel::StatsPeriod => Panel::Stats,
                    Panel::Stats => Panel::Broadcast,
                    Panel::Broadcast => Panel::Chat,
                    Panel::Chat => Panel::ChatInput,
                    Panel::ChatInput => Panel::UserInfo,
                };
                Outbound::None
            }
            KeyCode::Esc => Outbound::Quit,
            _ => Outbound::None,
        }
    }
}
