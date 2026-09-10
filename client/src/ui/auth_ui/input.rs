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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    fn type_str(app: &mut App, s: &str) {
        for c in s.chars() {
            app.handle_key(key(KeyCode::Char(c)));
        }
    }

    // --- Schermata di scelta ---

    #[test]
    fn frecce_alternano_login_e_registrazione() {
        let mut app = App::new();
        assert!(matches!(app.selected_action, AuthAction::Login));
        app.handle_key(key(KeyCode::Down));
        assert!(matches!(app.selected_action, AuthAction::Register));
        app.handle_key(key(KeyCode::Up));
        assert!(matches!(app.selected_action, AuthAction::Login));
    }

    #[test]
    fn invio_su_login_apre_il_form_di_login() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Enter));
        assert!(matches!(app.screen, Screen::Login));
        assert!(matches!(app.focus, Focus::Username));
    }

    #[test]
    fn invio_su_registrazione_apre_il_form_di_registrazione() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Down)); // seleziona Registrazione
        app.handle_key(key(KeyCode::Enter));
        assert!(matches!(app.screen, Screen::Register));
        assert!(matches!(app.focus, Focus::Username));
    }

    #[test]
    fn esc_sulla_scelta_restituisce_cancel() {
        let mut app = App::new();
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::Cancel));
    }

    // --- Form di login ---

    fn login_screen() -> App {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Enter)); // Login è la scelta di default
        app
    }

    #[test]
    fn tab_alterna_username_e_password_in_login() {
        let mut app = login_screen();
        assert!(matches!(app.focus, Focus::Username));
        app.handle_key(key(KeyCode::Tab));
        assert!(matches!(app.focus, Focus::Password));
        app.handle_key(key(KeyCode::Tab));
        assert!(matches!(app.focus, Focus::Username));
    }

    #[test]
    fn scrittura_va_nel_campo_con_il_focus_in_login() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");
        assert_eq!(app.username, "mario");
        assert_eq!(app.password, "supersegreta");
    }

    #[test]
    fn backspace_cancella_dal_campo_con_il_focus_in_login() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.username, "mari");
    }

    #[test]
    fn invio_con_campi_vuoti_in_login_da_errore() {
        let mut app = login_screen();
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert_eq!(
            app.error_message.as_deref(),
            Some("Username e password non possono essere vuoti.")
        );
        assert!(!app.awaiting_response);
    }

    #[test]
    fn invio_con_campi_validi_in_login_invia_le_credenziali() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");

        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::SendLogin { username, password } => {
                assert_eq!(username, "mario");
                assert_eq!(password, "supersegreta");
            }
            _ => panic!("atteso Outbound::SendLogin"),
        }
        assert!(app.awaiting_response);
        assert!(app.error_message.is_none());
    }

    #[test]
    fn esc_in_login_torna_alla_scelta_e_pulisce_i_campi() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");

        app.handle_key(key(KeyCode::Esc));
        assert!(matches!(app.screen, Screen::ChooseAction));
        assert!(app.username.is_empty());
        assert!(app.password.is_empty());
    }

    // --- Form di registrazione ---

    fn register_screen() -> App {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Down)); // seleziona Registrazione
        app.handle_key(key(KeyCode::Enter));
        app
    }

    #[test]
    fn tab_scorre_i_tre_campi_in_registrazione() {
        let mut app = register_screen();
        assert!(matches!(app.focus, Focus::Username));
        app.handle_key(key(KeyCode::Tab));
        assert!(matches!(app.focus, Focus::Password));
        app.handle_key(key(KeyCode::Tab));
        assert!(matches!(app.focus, Focus::ConfirmPassword));
        app.handle_key(key(KeyCode::Tab));
        assert!(matches!(app.focus, Focus::Username));
    }

    #[test]
    fn invio_con_campi_vuoti_in_registrazione_da_errore() {
        let mut app = register_screen();
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert_eq!(
            app.error_message.as_deref(),
            Some("Username e password non possono essere vuoti.")
        );
    }

    #[test]
    fn password_diverse_in_registrazione_danno_errore() {
        let mut app = register_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "altra-password");

        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert_eq!(
            app.error_message.as_deref(),
            Some("Le password non corrispondono.")
        );
        assert!(!app.awaiting_response);
    }

    #[test]
    fn invio_con_campi_validi_in_registrazione_invia_le_credenziali() {
        let mut app = register_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");

        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::SendRegister { username, password } => {
                assert_eq!(username, "mario");
                assert_eq!(password, "supersegreta");
            }
            _ => panic!("atteso Outbound::SendRegister"),
        }
        assert!(app.awaiting_response);
    }

    #[test]
    fn esc_in_registrazione_torna_alla_scelta_e_pulisce_i_campi() {
        let mut app = register_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Esc));
        assert!(matches!(app.screen, Screen::ChooseAction));
        assert!(app.username.is_empty());
    }

    // --- In attesa di risposta dal server ---

    #[test]
    fn mentre_si_attende_risposta_i_tasti_normali_sono_ignorati() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");
        app.handle_key(key(KeyCode::Enter)); // awaiting_response = true

        let before = app.username.clone();
        let result = app.handle_key(key(KeyCode::Char('x')));
        assert!(matches!(result, Outbound::None));
        assert_eq!(app.username, before); // il carattere non viene scritto da nessuna parte
    }

    #[test]
    fn esc_mentre_si_attende_risposta_annulla() {
        let mut app = login_screen();
        type_str(&mut app, "mario");
        app.handle_key(key(KeyCode::Tab));
        type_str(&mut app, "supersegreta");
        app.handle_key(key(KeyCode::Enter)); // awaiting_response = true

        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::Cancel));
        assert!(!app.awaiting_response);
    }
}
