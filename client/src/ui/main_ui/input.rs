use super::state::{App, Panel};
use common::protocol::TimePeriod;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    SendChat { message: String },
    QueryStats { period: TimePeriod },
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
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Panel::UserInfo => Panel::ChatInput,
                    Panel::Movement => Panel::UserInfo,
                    Panel::StatsPeriod => Panel::Movement,
                    Panel::Stats => Panel::StatsPeriod,
                    Panel::Broadcast => Panel::Stats,
                    Panel::Chat => Panel::Broadcast,
                    Panel::ChatInput => Panel::Chat,
                };
            }
            _ => {}
        }

        if self.focus == Panel::ChatInput {
            return self.handle_chat_input_key(key.code);
        }

        if self.focus == Panel::StatsPeriod {
            return self.handle_stats_period_key(key.code);
        }

        if self.focus == Panel::Chat {
            return self.handle_chat_scroll_key(key.code);
        }

        if self.focus == Panel::Broadcast {
            return self.handle_broadcast_scroll_key(key.code);
        }
        Outbound::None
    }

    fn handle_chat_input_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Char(c) => {
                self.chat_input.push(c);
                self.chat_input_scroll = 0; // torna a mostrare la fine del testo mentre digiti
                Outbound::None
            }
            KeyCode::Backspace => {
                self.chat_input.pop();
                self.chat_input_scroll = 0;
                Outbound::None
            }
            KeyCode::Up => {
                self.chat_input_scroll = self.chat_input_scroll.saturating_add(1);
                Outbound::None
            }
            KeyCode::Down => {
                self.chat_input_scroll = self.chat_input_scroll.saturating_sub(1);
                Outbound::None
            }
            KeyCode::Enter => {
                if self.chat_input.is_empty() {
                    return Outbound::None;
                }
                self.chat_input_scroll = 0;
                let message = std::mem::take(&mut self.chat_input);
                Outbound::SendChat { message }
            }
            _ => Outbound::None,
        }
    }

    fn handle_stats_period_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => {
                self.selected_period = match self.selected_period {
                    TimePeriod::Today => TimePeriod::ThisMonth,
                    TimePeriod::ThisMonth => TimePeriod::ThisWeek,
                    TimePeriod::ThisWeek => TimePeriod::Today,
                };
                Outbound::None
            }
            KeyCode::Down => {
                self.selected_period = match self.selected_period {
                    TimePeriod::Today => TimePeriod::ThisWeek,
                    TimePeriod::ThisWeek => TimePeriod::ThisMonth,
                    TimePeriod::ThisMonth => TimePeriod::Today,
                };
                Outbound::None
            }
            KeyCode::Enter => Outbound::QueryStats {
                period: self.selected_period.clone(),
            },
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
