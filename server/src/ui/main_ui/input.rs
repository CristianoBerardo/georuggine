use crate::ui::main_ui::state::ConnectedUsers;

use super::state::{App, Panel};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    // UserSelected { username: String },
    SendChat { message: String },
    SendBroadcast { message: String },
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
                    Panel::ChatInput => Panel::Chat,
                    Panel::Chat => Panel::Broadcast,
                    Panel::Broadcast => Panel::SelectUser,
                    Panel::SelectUser => Panel::Users,
                };
            }
            _ => {}
        }

        if self.focus == Panel::SelectUser {
            self.is_broadcast_mode = false;
            return self.handle_select_connected_user_key(key.code);
        }
        if self.focus == Panel::Broadcast {
            self.is_broadcast_mode = true;
            return self.handle_chat_broadcast_input_key(key.code);
        }
        if self.focus == Panel::Chat {
            self.is_broadcast_mode = false;
            return self.handle_chat_scroll_key(key.code);
        }

        if self.focus == Panel::ChatInput {
            self.is_broadcast_mode = false;
            return self.handle_chat_input_key(key.code);
        }
        Outbound::None
    }

    fn handle_select_connected_user_key(&mut self, code: KeyCode) -> Outbound {
        let len = self.connected_users.connected_users.len();
        if len == 0 {
            return Outbound::None;
        }

        match code {
            KeyCode::Up => {
                self.connected_users.index_selected =
                    Some(match self.connected_users.index_selected {
                        Some(0) | None => len - 1,
                        Some(i) => i - 1,
                    });
            }
            KeyCode::Down => {
                self.connected_users.index_selected =
                    Some(match self.connected_users.index_selected {
                        None => 0,
                        Some(i) if i >= len - 1 => 0,
                        Some(i) => i + 1,
                    });
            }
            _ => return Outbound::None,
        }

        // if let Some(idx) = self.connected_users.index_selected {
        //     self.selected_user = self.connected_users.connected_users[idx].username.clone();
        // }
        Outbound::None
    }

    fn handle_chat_input_key(&mut self, code: KeyCode) -> Outbound {
        if self.connected_users.index_selected.is_none() {
            return Outbound::None;
        }

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

    fn handle_chat_broadcast_input_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Char(c) => {
                self.broadcast_chat_input.push(c);
                Outbound::None
            }
            KeyCode::Backspace => {
                self.broadcast_chat_input.pop();
                Outbound::None
            }
            KeyCode::Enter => {
                if self.broadcast_chat_input.is_empty() {
                    return Outbound::None;
                }
                let message = std::mem::take(&mut self.broadcast_chat_input);
                Outbound::SendBroadcast { message }
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
