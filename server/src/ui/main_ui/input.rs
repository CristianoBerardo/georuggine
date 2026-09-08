use super::state::{App, Panel, StatsStep};
use common::protocol::TimePeriod;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(crate) enum Outbound {
    Quit,
    SendChat { message: String },
    SendBroadcast { message: String },
    QueryStats { username: String, period: TimePeriod },
    None,
}

impl App {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        // Nel secondo passo (scelta periodo) Backspace torna alla scelta
        // dell'utente: Esc resta riservato solo all'uscita dall'applicazione.
        if self.focus == Panel::StatsSelect
            && self.stats_step == StatsStep::SelectPeriod
            && key.code == KeyCode::Backspace
        {
            self.stats_step = StatsStep::SelectUser;
            return Outbound::None;
        }

        match key.code {
            KeyCode::Esc => return Outbound::Quit,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panel::Users => Panel::BroadcastChat,
                    Panel::BroadcastChat => Panel::Broadcast,
                    Panel::Broadcast => Panel::SelectUser,
                    Panel::SelectUser => Panel::Chat,
                    Panel::Chat => Panel::ChatInput,
                    Panel::ChatInput => Panel::StatsSelect,
                    Panel::StatsSelect => Panel::ErrorLog,
                    Panel::ErrorLog => Panel::Users,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Panel::Users => Panel::ErrorLog,
                    Panel::ErrorLog => Panel::StatsSelect,
                    Panel::StatsSelect => Panel::ChatInput,
                    Panel::ChatInput => Panel::Chat,
                    Panel::Chat => Panel::SelectUser,
                    Panel::SelectUser => Panel::Broadcast,
                    Panel::Broadcast => Panel::BroadcastChat,
                    Panel::BroadcastChat => Panel::Users,
                };
            }
            _ => {}
        }

        if self.focus == Panel::StatsSelect {
            return self.handle_stats_select_key(key.code);
        }
        if self.focus == Panel::Users {
            return self.handle_users_scroll_key(key.code);
        }
        if self.focus == Panel::SelectUser {
            return self.handle_select_connected_user_key(key.code);
        }
        if self.focus == Panel::Broadcast {
            return self.handle_chat_broadcast_input_key(key.code);
        }
        if self.focus == Panel::BroadcastChat {
            return self.handle_broadcast_scroll_key(key.code);
        }
        if self.focus == Panel::Chat {
            return self.handle_chat_scroll_key(key.code);
        }

        if self.focus == Panel::ChatInput {
            return self.handle_chat_input_key(key.code);
        }
        if self.focus == Panel::ErrorLog {
            return self.handle_error_scroll_key(key.code);
        }
        Outbound::None
    }

    fn handle_users_scroll_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => self.users_scroll = self.users_scroll.saturating_add(1),
            KeyCode::Down => self.users_scroll = self.users_scroll.saturating_sub(1),
            _ => {}
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

        if let Some(idx) = self.connected_users.index_selected {
            self.connected_users.connected_users[idx].unread_count = 0;
        }

        Outbound::None
    }

    fn handle_chat_input_key(&mut self, code: KeyCode) -> Outbound {
        if self.connected_users.index_selected.is_none() {
            return Outbound::None;
        }

        match code {
            KeyCode::Char(c) => {
                self.chat_input.push(c);
                self.chat_input_scroll = 0;
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

    fn handle_chat_broadcast_input_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Char(c) => {
                self.broadcast_chat_input.push(c);
                self.broadcast_input_scroll = 0;
                Outbound::None
            }
            KeyCode::Backspace => {
                self.broadcast_chat_input.pop();
                self.broadcast_input_scroll = 0;
                Outbound::None
            }
            KeyCode::Up => {
                self.broadcast_input_scroll = self.broadcast_input_scroll.saturating_add(1);
                Outbound::None
            }
            KeyCode::Down => {
                self.broadcast_input_scroll = self.broadcast_input_scroll.saturating_sub(1);
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

    fn handle_error_scroll_key(&mut self, code: KeyCode) -> Outbound {
        match code {
            KeyCode::Up => self.error_scroll = self.error_scroll.saturating_add(1),
            KeyCode::Down => self.error_scroll = self.error_scroll.saturating_sub(1),
            _ => {}
        }
        Outbound::None
    }

    fn handle_stats_select_key(&mut self, code: KeyCode) -> Outbound {
        match self.stats_step {
            StatsStep::SelectUser => {
                let len = self.users.len();
                if len == 0 {
                    return Outbound::None;
                }
                match code {
                    KeyCode::Up => {
                        self.stats_user_index = Some(match self.stats_user_index {
                            Some(0) | None => len - 1,
                            Some(i) => i - 1,
                        });
                    }
                    KeyCode::Down => {
                        self.stats_user_index = Some(match self.stats_user_index {
                            None => 0,
                            Some(i) if i >= len - 1 => 0,
                            Some(i) => i + 1,
                        });
                    }
                    KeyCode::Enter => {
                        if self.stats_user_index.is_none() {
                            self.stats_user_index = Some(0);
                        }
                        self.stats_step = StatsStep::SelectPeriod;
                    }
                    _ => {}
                }
                Outbound::None
            }
            StatsStep::SelectPeriod => match code {
                KeyCode::Up => {
                    self.stats_period = match self.stats_period {
                        TimePeriod::Today => TimePeriod::ThisMonth,
                        TimePeriod::ThisMonth => TimePeriod::ThisWeek,
                        TimePeriod::ThisWeek => TimePeriod::Today,
                    };
                    Outbound::None
                }
                KeyCode::Down => {
                    self.stats_period = match self.stats_period {
                        TimePeriod::Today => TimePeriod::ThisWeek,
                        TimePeriod::ThisWeek => TimePeriod::ThisMonth,
                        TimePeriod::ThisMonth => TimePeriod::Today,
                    };
                    Outbound::None
                }
                KeyCode::Enter => {
                    let idx = self.stats_user_index.unwrap_or(0);
                    match self.users.get(idx) {
                        Some(user) => Outbound::QueryStats {
                            username: user.username.clone(),
                            period: self.stats_period.clone(),
                        },
                        None => Outbound::None,
                    }
                }
                _ => Outbound::None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::{ConnectedUsers, UserChat, Users};
    use crate::state::UserStatus;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    fn registered_users(usernames: &[&str]) -> Vec<Users> {
        usernames
            .iter()
            .map(|&username| Users {
                username: username.to_string(),
                status: UserStatus::Sconnesso,
            })
            .collect()
    }

    fn connected(usernames: &[&str]) -> ConnectedUsers {
        ConnectedUsers {
            connected_users: usernames
                .iter()
                .map(|&username| UserChat {
                    username: username.to_string(),
                    chat_log: Vec::new(),
                    unread_count: 0,
                })
                .collect(),
            index_selected: None,
        }
    }

    // --- Tab / BackTab ---

    #[test]
    fn tab_scorre_i_riquadri_in_ordine_e_torna_al_primo() {
        let mut app = App::test_default();
        let expected = [
            Panel::BroadcastChat,
            Panel::Broadcast,
            Panel::SelectUser,
            Panel::Chat,
            Panel::ChatInput,
            Panel::StatsSelect,
            Panel::ErrorLog,
            Panel::Users, // dopo l'ultimo, si torna al primo
        ];
        for expected_panel in expected {
            app.handle_key(key(KeyCode::Tab));
            assert!(app.focus == expected_panel);
        }
    }

    #[test]
    fn backtab_scorre_allindietro() {
        let mut app = App::test_default();
        app.handle_key(key(KeyCode::BackTab));
        assert!(app.focus == Panel::ErrorLog);
    }

    #[test]
    fn esc_restituisce_quit_da_qualunque_riquadro() {
        let mut app = App::test_default();
        let result = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, Outbound::Quit));
    }

    // --- Utenti registrati ---

    #[test]
    fn frecce_in_utenti_registrati_scorrono_la_lista() {
        let mut app = App::test_default();
        app.focus = Panel::Users;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.users_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.users_scroll, 0);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.users_scroll, 0); // non va sotto zero
    }

    // --- Selezione utenti collegati ---

    #[test]
    fn senza_utenti_connessi_le_frecce_non_fanno_nulla() {
        let mut app = App::test_default();
        app.focus = Panel::SelectUser;
        app.handle_key(key(KeyCode::Down));
        assert!(app.connected_users.index_selected.is_none());
    }

    #[test]
    fn giu_seleziona_il_primo_poi_scorre_e_torna_in_cima() {
        let mut app = App::test_default();
        app.focus = Panel::SelectUser;
        app.connected_users = connected(&["anna", "mario"]);

        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.connected_users.index_selected, Some(0));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.connected_users.index_selected, Some(1));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.connected_users.index_selected, Some(0)); // torna al primo
    }

    #[test]
    fn su_scorre_allindietro_e_avvolge_dallalto() {
        let mut app = App::test_default();
        app.focus = Panel::SelectUser;
        app.connected_users = connected(&["anna", "mario"]);

        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.connected_users.index_selected, Some(1)); // nessuna selezione -> ultimo
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.connected_users.index_selected, Some(0));
    }

    #[test]
    fn selezionare_un_utente_azzera_i_suoi_messaggi_non_letti() {
        let mut app = App::test_default();
        app.focus = Panel::SelectUser;
        app.connected_users = connected(&["anna", "mario"]);
        app.connected_users.connected_users[1].unread_count = 3;

        app.handle_key(key(KeyCode::Down)); // seleziona anna (indice 0)
        app.handle_key(key(KeyCode::Down)); // seleziona mario (indice 1)

        assert_eq!(app.connected_users.connected_users[1].unread_count, 0);
    }

    // --- Scrivi messaggio Broadcast ---

    #[test]
    fn scrivere_in_broadcast_accumula_testo() {
        let mut app = App::test_default();
        app.focus = Panel::Broadcast;
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('i')));
        app.handle_key(key(KeyCode::Char('a')));
        assert_eq!(app.broadcast_chat_input, "cia");
    }

    #[test]
    fn invio_broadcast_vuoto_non_fa_nulla() {
        let mut app = App::test_default();
        app.focus = Panel::Broadcast;
        let result = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, Outbound::None));
    }

    #[test]
    fn invio_broadcast_lo_svuota_e_lo_restituisce() {
        let mut app = App::test_default();
        app.focus = Panel::Broadcast;
        app.broadcast_chat_input = "ciao a tutti".to_string();
        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::SendBroadcast { message } => assert_eq!(message, "ciao a tutti"),
            _ => panic!("atteso Outbound::SendBroadcast"),
        }
        assert!(app.broadcast_chat_input.is_empty());
    }

    #[test]
    fn frecce_in_broadcast_input_scorrono_il_messaggio() {
        let mut app = App::test_default();
        app.focus = Panel::Broadcast;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.broadcast_input_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.broadcast_input_scroll, 0);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.broadcast_input_scroll, 0); // non va sotto zero
    }

    // --- Scrivi messaggio (diretto) ---

    #[test]
    fn senza_utente_selezionato_non_si_puo_scrivere() {
        let mut app = App::test_default();
        app.focus = Panel::ChatInput;
        app.handle_key(key(KeyCode::Char('c')));
        assert!(app.chat_input.is_empty());
    }

    #[test]
    fn con_un_utente_selezionato_si_puo_scrivere() {
        let mut app = App::test_default();
        app.focus = Panel::ChatInput;
        app.connected_users = connected(&["mario"]);
        app.connected_users.index_selected = Some(0);

        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('i')));
        app.handle_key(key(KeyCode::Char('a')));
        assert_eq!(app.chat_input, "cia");
    }

    #[test]
    fn invio_messaggio_diretto_lo_svuota_e_lo_restituisce() {
        let mut app = App::test_default();
        app.focus = Panel::ChatInput;
        app.connected_users = connected(&["mario"]);
        app.connected_users.index_selected = Some(0);
        app.chat_input = "ciao".to_string();

        let result = app.handle_key(key(KeyCode::Enter));
        match result {
            Outbound::SendChat { message } => assert_eq!(message, "ciao"),
            _ => panic!("atteso Outbound::SendChat"),
        }
        assert!(app.chat_input.is_empty());
    }

    #[test]
    fn frecce_in_chat_input_scorrono_il_messaggio_se_un_utente_e_selezionato() {
        let mut app = App::test_default();
        app.focus = Panel::ChatInput;
        app.connected_users = connected(&["mario"]);
        app.connected_users.index_selected = Some(0);

        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.chat_input_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_input_scroll, 0);
    }

    // --- Scroll di chat, broadcast chat ed errori ---

    #[test]
    fn frecce_in_chat_scorrono_lo_storico() {
        let mut app = App::test_default();
        app.focus = Panel::Chat;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.chat_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.chat_scroll, 0);
    }

    #[test]
    fn frecce_in_broadcast_chat_scorrono_lo_storico() {
        let mut app = App::test_default();
        app.focus = Panel::BroadcastChat;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.broadcast_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.broadcast_scroll, 0);
    }

    #[test]
    fn frecce_in_error_log_scorrono_lo_storico() {
        let mut app = App::test_default();
        app.focus = Panel::ErrorLog;
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.error_scroll, 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.error_scroll, 0);
    }

    // --- Statistiche (scelta utente + periodo) ---

    #[test]
    fn senza_utenti_registrati_le_frecce_non_fanno_nulla() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.handle_key(key(KeyCode::Down));
        assert!(app.stats_user_index.is_none());
    }

    #[test]
    fn giu_seleziona_il_primo_poi_scorre_e_avvolge() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna", "mario"]);

        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.stats_user_index, Some(0));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.stats_user_index, Some(1));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.stats_user_index, Some(0)); // avvolge dal fondo
    }

    #[test]
    fn su_in_statistiche_scorre_allindietro_e_avvolge_dallalto() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna", "mario"]);

        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.stats_user_index, Some(1)); // dal "nessuna selezione" va all'ultimo
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.stats_user_index, Some(0));
    }

    #[test]
    fn invio_su_utente_selezionato_passa_alla_scelta_periodo() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna", "mario"]);
        app.stats_user_index = Some(1);

        app.handle_key(key(KeyCode::Enter));

        assert!(matches!(app.stats_step, StatsStep::SelectPeriod));
        assert_eq!(app.stats_user_index, Some(1)); // la selezione non cambia
    }

    #[test]
    fn invio_senza_selezione_esplicita_usa_il_primo_utente() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna", "mario"]);

        app.handle_key(key(KeyCode::Enter));

        assert!(matches!(app.stats_step, StatsStep::SelectPeriod));
        assert_eq!(app.stats_user_index, Some(0));
    }

    #[test]
    fn frecce_in_selezione_periodo_cambiano_periodo() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna"]);
        app.stats_step = StatsStep::SelectPeriod;
        assert!(matches!(app.stats_period, TimePeriod::Today));

        app.handle_key(key(KeyCode::Down));
        assert!(matches!(app.stats_period, TimePeriod::ThisWeek));

        app.handle_key(key(KeyCode::Down));
        assert!(matches!(app.stats_period, TimePeriod::ThisMonth));

        app.handle_key(key(KeyCode::Up));
        assert!(matches!(app.stats_period, TimePeriod::ThisWeek));
    }

    #[test]
    fn backspace_in_selezione_periodo_torna_alla_scelta_utente() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna"]);
        app.stats_step = StatsStep::SelectPeriod;

        let result = app.handle_key(key(KeyCode::Backspace));

        assert!(matches!(result, Outbound::None));
        assert!(matches!(app.stats_step, StatsStep::SelectUser));
        assert!(matches!(app.focus, Panel::StatsSelect)); // resta sullo stesso riquadro
    }

    #[test]
    fn invio_in_selezione_periodo_restituisce_query_per_lutente_scelto() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.users = registered_users(&["anna", "mario"]);
        app.stats_step = StatsStep::SelectPeriod;
        app.stats_user_index = Some(1);

        let result = app.handle_key(key(KeyCode::Enter));

        match result {
            Outbound::QueryStats { username, period } => {
                assert_eq!(username, "mario");
                assert!(matches!(period, TimePeriod::Today));
            }
            _ => panic!("atteso Outbound::QueryStats"),
        }
    }

    #[test]
    fn invio_in_selezione_periodo_senza_utenti_non_restituisce_nulla() {
        let mut app = App::test_default();
        app.focus = Panel::StatsSelect;
        app.stats_step = StatsStep::SelectPeriod;
        app.stats_user_index = Some(0); // indice ormai orfano: nessun utente in lista

        let result = app.handle_key(key(KeyCode::Enter));

        assert!(matches!(result, Outbound::None));
    }
}
