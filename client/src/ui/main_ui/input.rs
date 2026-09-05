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
            Panel::Movement,
            Panel::StatsPeriod,
            Panel::Stats,
            Panel::DeleteAccount,
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

    // --- Periodo statistiche ---

    #[test]
    fn frecce_in_periodo_statistiche_cambiano_periodo() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::StatsPeriod;
        assert!(matches!(app.selected_period, TimePeriod::Today));

        app.handle_key(key(KeyCode::Down));
        assert!(matches!(app.selected_period, TimePeriod::ThisWeek));

        app.handle_key(key(KeyCode::Down));
        assert!(matches!(app.selected_period, TimePeriod::ThisMonth));

        app.handle_key(key(KeyCode::Up));
        assert!(matches!(app.selected_period, TimePeriod::ThisWeek));
    }

    #[test]
    fn invio_in_periodo_statistiche_restituisce_query() {
        let mut app = App::new("mario".to_string());
        app.focus = Panel::StatsPeriod;
        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::QueryStats { period } => assert!(matches!(period, TimePeriod::Today)),
            _ => panic!("atteso Outbound::QueryStats"),
        }
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
}
