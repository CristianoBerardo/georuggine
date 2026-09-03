use common::protocol::AuthAction;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::state::{App, Focus, Screen};

pub(crate) enum Outbound {
    SendLogin { username: String, password: String },
    SendRegister { username: String, password: String },
    Cancel,
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente in base allo schermo attuale e al focus
        if self.awaiting_response {
            if key.code == KeyCode::Esc {
                self.awaiting_response = false;
                self.error_message = None;
                self.info_message = None;
                return Outbound::Cancel;
            } else {
                return Outbound::None;
            }
        }

        match self.screen {
            Screen::ChooseAction => self.handle_selected_action_key(key),
            Screen::Login => self.handle_login_key(key),
            Screen::Register => self.handle_register_key(key),
        }
    }

    fn handle_selected_action_key(&mut self, key: KeyEvent) -> Outbound {
        if key.code == KeyCode::Up || key.code == KeyCode::Down {
            // Toggle dell'azione selezionata tra login e registrazione
            self.selected_action = match self.selected_action {
                AuthAction::Login => AuthAction::Register,
                AuthAction::Register => AuthAction::Login,
            };
        } else if key.code == KeyCode::Enter {
            self.screen = match self.selected_action {
                AuthAction::Login => {
                    self.focus = Focus::Username;
                    self.error_message = None;
                    self.info_message = None;
                    Screen::Login
                }
                AuthAction::Register => {
                    self.focus = Focus::Username;
                    self.error_message = None;
                    self.info_message = None;
                    Screen::Register
                }
            };
        } else if key.code == KeyCode::Esc {
            self.awaiting_response = false;
            // Gestisci l'uscita dal menu di scelta
            return Outbound::Cancel;
        }
        Outbound::None
    }

    fn handle_login_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente nel form di login
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Username => Focus::Password,
                    Focus::Password => Focus::Username,
                    _ => Focus::Username,
                }
            }
            KeyCode::Char(c) => match self.focus {
                Focus::Username => self.username.push(c),
                Focus::Password => self.password.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                Focus::Username => {
                    self.username.pop();
                }
                Focus::Password => {
                    self.password.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error_message =
                        Some("Username e password non possono essere vuoti.".to_string());
                    return Outbound::None;
                } else {
                    self.error_message = None;
                    self.awaiting_response = true;
                    return Outbound::SendLogin {
                        username: self.username.clone(),
                        password: self.password.clone(),
                    };
                }
            }
            KeyCode::Esc => {
                self.screen = Screen::ChooseAction;
                self.focus = Focus::Username;
                self.username = String::new();
                self.password = String::new();
                self.confirm_password = String::new();
                self.error_message = None;
                self.info_message = None;
                return Outbound::None;
            }
            _ => {
                return Outbound::None;
            }
        }
        Outbound::None
    }

    fn handle_register_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente nel form di registrazione
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Username => Focus::Password,
                    Focus::Password => Focus::ConfirmPassword,
                    Focus::ConfirmPassword => Focus::Username,
                }
            }
            KeyCode::Char(c) => match self.focus {
                Focus::Username => self.username.push(c),
                Focus::Password => self.password.push(c),
                Focus::ConfirmPassword => self.confirm_password.push(c),
            },
            KeyCode::Backspace => match self.focus {
                Focus::Username => {
                    self.username.pop();
                }
                Focus::Password => {
                    self.password.pop();
                }
                Focus::ConfirmPassword => {
                    self.confirm_password.pop();
                }
            },
            KeyCode::Enter => {
                if self.username.is_empty()
                    || self.password.is_empty()
                    || self.confirm_password.is_empty()
                {
                    self.error_message =
                        Some("Username e password non possono essere vuoti.".to_string());
                    return Outbound::None;
                } else if self.password != self.confirm_password {
                    self.error_message = Some("Le password non corrispondono.".to_string());
                    return Outbound::None;
                } else {
                    self.error_message = None;
                    self.awaiting_response = true;
                    return Outbound::SendRegister {
                        username: self.username.clone(),
                        password: self.password.clone(),
                    };
                }
            }
            KeyCode::Esc => {
                self.screen = Screen::ChooseAction;
                self.focus = Focus::Username;
                self.username = String::new();
                self.password = String::new();
                self.confirm_password = String::new();
                self.error_message = None;
                self.info_message = None;
                return Outbound::None;
            }
            _ => {
                return Outbound::None;
            }
        }
        Outbound::None
    }
}
