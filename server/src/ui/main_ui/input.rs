use crate::ui::main_ui::state::Users;

use super::state::{App, Panel};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    UserSelected { username: String },
    SendChat { message: String },
    // RequestStats(TimePeriod),
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        match key.code {
            KeyCode::Esc => return Outbound::Quit,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panel::Users => Panel::SelectUser,
                    Panel::SelectUser => Panel::Broadcast,
                    Panel::Broadcast => Panel::Chat,
                    Panel::Chat => Panel::ChatInput,
                    Panel::ChatInput => Panel::Users,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Panel::Users => Panel::ChatInput,
                    Panel::SelectUser => Panel::Users,
                    Panel::Broadcast => Panel::SelectUser,
                    Panel::Chat => Panel::Broadcast,
                    Panel::ChatInput => Panel::Chat,
                };
            }
            _ => {}
        }

        if self.focus == Panel::SelectUser {
            return self.handle_select_user_key(key.code);
        }
        if self.focus == Panel::Broadcast {
            return self.handle_broadcast_scroll_key(key.code);
        }
        if self.focus == Panel::Chat {
            return self.handle_chat_scroll_key(key.code);
        }

        if self.focus == Panel::ChatInput {
            return self.handle_chat_input_key(key.code);
        }
        Outbound::None
    }

    fn handle_select_user_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => {
                self.selected_user = if self.users.index_selected == 0 {
                    self.users.index_selected = self.users.users.len() - 1;
                    self.users.users[self.users.users.len() - 1].clone()
                } else {
                    self.users.users[self.users.index_selected - 1].clone()
                };
                self.users.index_selected = self.users.index_selected.saturating_sub(1);
                Outbound::None
            }
            KeyCode::Down => {
                self.selected_user = if self.users.index_selected == self.users.users.len() - 1 {
                    self.users.index_selected = 0;
                    self.users.users[0].clone()
                } else {
                    self.users.users[self.users.index_selected + 1].clone()
                };
                self.users.index_selected = self.users.index_selected + 1;
                Outbound::None
            }
            KeyCode::Enter => Outbound::UserSelected {
                username: self.selected_user.clone(),
            },
            _ => Outbound::None,
        }
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

    fn handle_chat_scroll_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => self.chat_scroll = self.chat_scroll.saturating_add(1),
            KeyCode::Down => self.chat_scroll = self.chat_scroll.saturating_sub(1),
            _ => {}
        }
        Outbound::None
    }

    fn handle_broadcast_scroll_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => self.broadcast_scroll = self.broadcast_scroll.saturating_add(1),
            KeyCode::Down => self.broadcast_scroll = self.broadcast_scroll.saturating_sub(1),
            _ => {}
        }
        Outbound::None
    }
}
