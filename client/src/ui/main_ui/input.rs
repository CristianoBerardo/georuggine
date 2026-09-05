use super::state::{App, DeleteAccountStep, Panel};
use common::protocol::TimePeriod;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    SendChat { message: String },
    QueryStats { period: TimePeriod },
    DeleteAccount { password: String },
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        // Il flusso di eliminazione account, una volta avviato, cattura tutti i
        // tasti (compreso Esc, che altrimenti chiuderebbe l'applicazione).
        if self.focus == Panel::DeleteAccount && self.delete_step != DeleteAccountStep::Idle {
            return self.handle_delete_account_active_key(key.code);
        }

        match key.code {
            KeyCode::Esc => return Outbound::Quit,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panel::UserInfo => Panel::Movement,
                    Panel::Movement => Panel::StatsPeriod,
                    Panel::StatsPeriod => Panel::Stats,
                    Panel::Stats => Panel::DeleteAccount,
                    Panel::DeleteAccount => Panel::Broadcast,
                    Panel::Broadcast => Panel::Chat,
                    Panel::Chat => Panel::ChatInput,
                    Panel::ChatInput => Panel::ErrorLog,
                    Panel::ErrorLog => Panel::UserInfo,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Panel::UserInfo => Panel::ErrorLog,
                    Panel::ErrorLog => Panel::ChatInput,
                    Panel::ChatInput => Panel::Chat,
                    Panel::Chat => Panel::Broadcast,
                    Panel::Broadcast => Panel::DeleteAccount,
                    Panel::DeleteAccount => Panel::Stats,
                    Panel::Movement => Panel::UserInfo,
                    Panel::StatsPeriod => Panel::Movement,
                    Panel::Stats => Panel::StatsPeriod,
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
        if self.focus == Panel::ErrorLog {
            return self.handle_error_scroll_key(key.code);
        }

        if self.focus == Panel::DeleteAccount {
            return self.handle_delete_account_key(key.code);
        }
        Outbound::None
    }

    /// Pannello "Elimina account" a riposo: Invio avvia il flusso (password + conferma).
    fn handle_delete_account_key(&mut self, code: KeyCode) -> Outbound {
        if code == KeyCode::Enter {
            self.delete_step = DeleteAccountStep::EnterPassword;
            self.delete_password.clear();
            self.delete_error = None;
        }
        Outbound::None
    }

    /// Flusso di eliminazione account già avviato: richiesta password, poi conferma.
    fn handle_delete_account_active_key(&mut self, code: KeyCode) -> Outbound {
        if self.delete_pending {
            // In attesa della risposta del server: ignora l'input.
            return Outbound::None;
        }

        match self.delete_step {
            DeleteAccountStep::EnterPassword => match code {
                KeyCode::Char(c) => {
                    self.delete_password.push(c);
                    Outbound::None
                }
                KeyCode::Backspace => {
                    self.delete_password.pop();
                    Outbound::None
                }
                KeyCode::Enter => {
                    if self.delete_password.is_empty() {
                        self.delete_error = Some("Inserisci la password.".to_string());
                        return Outbound::None;
                    }
                    self.delete_error = None;
                    self.delete_step = DeleteAccountStep::Confirm;
                    Outbound::None
                }
                KeyCode::Esc => {
                    self.delete_step = DeleteAccountStep::Idle;
                    self.delete_password.clear();
                    self.delete_error = None;
                    Outbound::None
                }
                _ => Outbound::None,
            },
            DeleteAccountStep::Confirm => match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.delete_pending = true;
                    let password = std::mem::take(&mut self.delete_password);
                    Outbound::DeleteAccount { password }
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.delete_step = DeleteAccountStep::Idle;
                    self.delete_password.clear();
                    self.delete_error = None;
                    Outbound::None
                }
                _ => Outbound::None,
            },
            DeleteAccountStep::Idle => Outbound::None,
        }
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

    fn handle_error_scroll_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => self.error_scroll = self.error_scroll.saturating_add(1),
            KeyCode::Down => self.error_scroll = self.error_scroll.saturating_sub(1),
            _ => {}
        }
        Outbound::None
    }
}
