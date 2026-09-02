use super::state::{App, Panel};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    SendChat { message: String },
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        match key.code {
            KeyCode::Esc => return Outbound::Quit,
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
            }
            _ => {}
        }

        if self.focus == Panel::ChatInput {
            return self.handle_chat_input_key(key.code);
        }
        Outbound::None
    }

    fn handle_chat_input_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Char(c) => {
                self.chat_input.push(c);
                Outbound::None
            }
            KeyCode::Backspace => {
                self.chat_input.pop();
                Outbound::None
            }
            KeyCode::Enter => {
                if self.chat_input.is_empty() {
                    return Outbound::None;
                }
                let message = std::mem::take(&mut self.chat_input);
                Outbound::SendChat { message }
            }
            _ => Outbound::None,
        }
    }
}
