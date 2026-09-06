use super::state::{App, DeleteAccountStep, Panel};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    SendChat { message: String },
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
                    Panel::UserInfo => Panel::DeleteAccount,
                    Panel::DeleteAccount => Panel::Movement,
                    Panel::Movement => Panel::Broadcast,
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
                    Panel::Broadcast => Panel::Movement,
                    Panel::Movement => Panel::DeleteAccount,
                    Panel::DeleteAccount => Panel::UserInfo,
                };
            }
            _ => {}
        }

        if self.focus == Panel::ChatInput {
            return self.handle_chat_input_key(key.code);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    // --- Tab / BackTab ---

    #[test]
    fn tab_scorre_i_riquadri_in_ordine_e_torna_al_primo() {
        let mut app = App::new("mario".to_string());
        let expected = [
            Panel::DeleteAccount,
            Panel::Movement,
            Panel::Broadcast,
            Panel::Chat,
            Panel::ChatInput,
            Panel::ErrorLog,
            Panel::UserInfo, // dopo l'ultimo, si torna al primo
        ];
        for expected_panel in expected {
            app.handle_key(key(KeyCode::Tab));
            assert!(app.focus == expected_panel);
        }
    }

    #[test]
    fn backtab_scorre_allindietro() {
        let mut app = App::new("mario".to_string());
        app.handle_key(key(KeyCode::BackTab));
        assert!(app.focus == Panel::ErrorLog);
    }

    #[test]
    fn esc_restituisce_quit_da_qualunque_riquadro() {
        let mut app = App::new("mario".to_string());
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::Quit));
    }

    // --- Scrivi messaggio ---

    #[test]
    fn scrivere_in_chat_input_accumula_testo() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('i')));
        app.handle_key(key(KeyCode::Char('a')));
        assert_eq!(app.chat_input, "cia");
    }

    #[test]
    fn backspace_cancella_ultimo_carattere() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        app.chat_input = "ciao".to_string();
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.chat_input, "cia");
    }

    #[test]
    fn invio_messaggio_vuoto_non_fa_nulla() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert!(app.chat_input.is_empty());
    }

    #[test]
    fn invio_messaggio_lo_svuota_e_lo_restituisce() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        app.chat_input = "ciao".to_string();
        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::SendChat { message } => assert_eq!(message, "ciao"),
            _ => panic!("atteso Outbound::SendChat"),
        }
        assert!(app.chat_input.is_empty());
    }

    #[test]
    fn frecce_in_chat_input_scorrono_il_messaggio() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.chat_input_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_input_scroll, 0);
        // non deve andare sotto zero
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_input_scroll, 0);
    }

    #[test]
    fn digitare_resetta_lo_scroll_del_messaggio() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::ChatInput;
        app.chat_input_scroll = 5;
        app.handle_key(key(KeyCode::Char('a')));
        assert_eq!(app.chat_input_scroll, 0);
    }

    // --- Scroll di chat e broadcast ---

    #[test]
    fn frecce_in_chat_scorrono_lo_storico() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::Chat;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.chat_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_scroll, 0);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_scroll, 0); // non va sotto zero
    }

    #[test]
    fn frecce_in_broadcast_scorrono_lo_storico() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::Broadcast;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.broadcast_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.broadcast_scroll, 0);
    }

    // --- Elimina account ---

    #[test]
    fn invio_sul_riquadro_a_riposo_avvia_linserimento_password() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert!(matches!(app.delete_step, DeleteAccountStep::EnterPassword));
        assert!(app.delete_password.is_empty());
    }

    #[test]
    fn scrivere_la_password_la_accumula() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::EnterPassword;
        app.handle_key(key(KeyCode::Char('a')));
        app.handle_key(key(KeyCode::Char('b')));
        app.handle_key(key(KeyCode::Char('c')));
        assert_eq!(app.delete_password, "abc");
    }

    #[test]
    fn backspace_cancella_ultimo_carattere_della_password() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::EnterPassword;
        app.delete_password = "abc".to_string();
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.delete_password, "ab");
    }

    #[test]
    fn invio_con_password_vuota_da_errore_e_non_avanza() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::EnterPassword;
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
        assert!(matches!(app.delete_step, DeleteAccountStep::EnterPassword));
        assert!(app.delete_error.is_some());
    }

    #[test]
    fn invio_con_password_non_vuota_passa_alla_conferma() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::EnterPassword;
        app.delete_password = "segreta".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(matches!(app.delete_step, DeleteAccountStep::Confirm));
    }

    #[test]
    fn esc_durante_inserimento_password_annulla_senza_uscire() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::EnterPassword;
        app.delete_password = "segreta".to_string();
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::None)); // non Outbound::Quit
        assert!(matches!(app.delete_step, DeleteAccountStep::Idle));
        assert!(app.delete_password.is_empty());
    }

    #[test]
    fn y_in_conferma_restituisce_delete_account_e_svuota_la_password() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::Confirm;
        app.delete_password = "segreta".to_string();
        let result = app.handle_key(key(KeyCode::Char('y')));
        match result {
            Outbound::DeleteAccount { password } => assert_eq!(password, "segreta"),
            _ => panic!("atteso Outbound::DeleteAccount"),
        }
        assert!(app.delete_pending);
        assert!(app.delete_password.is_empty());
    }

    #[test]
    fn n_in_conferma_annulla_e_torna_a_riposo() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::Confirm;
        app.delete_password = "segreta".to_string();
        let result = app.handle_key(key(KeyCode::Char('n')));
        assert!(matches!(result, Outbound::None));
        assert!(matches!(app.delete_step, DeleteAccountStep::Idle));
        assert!(app.delete_password.is_empty());
        assert!(!app.delete_pending);
    }

    #[test]
    fn esc_in_conferma_annulla_come_n() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::Confirm;
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::None));
        assert!(matches!(app.delete_step, DeleteAccountStep::Idle));
    }

    #[test]
    fn durante_lattesa_della_risposta_i_tasti_sono_ignorati() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::DeleteAccount;
        app.delete_step = DeleteAccountStep::Confirm;
        app.delete_pending = true;
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::None));
        // Lo stato non cambia: nessuna uscita, nessun ritorno a riposo
        assert!(matches!(app.delete_step, DeleteAccountStep::Confirm));
        assert!(app.delete_pending);
    }
}
