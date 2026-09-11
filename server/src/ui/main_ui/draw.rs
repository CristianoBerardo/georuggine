use super::state::{App, StatsStep};
use crate::ui::size_control::size_too_small;
use common::protocol::TimePeriod;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        if size_too_small(frame.area()) {
            let warning = Paragraph::new("La dimensione del terminale è troppo piccola. Ridimensiona il terminale per continuare.")
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Attenzione"))
                .wrap(Wrap { trim: false });
            frame.render_widget(warning, frame.area());
            return;
        }

        // root[0] AREA PRINCIPALE (le due colonne)
        // root[1] LOG ERRORI (a tutta larghezza)
        // root[2] AREA DI AIUTO
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(root[0]);

        // Colonna 1: utenti registrati + broadcast (log e input)
        let col1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // utenti registrati
                Constraint::Min(6),    // broadcast chat (log)
                Constraint::Length(5), // input messaggio broadcast
            ])
            .split(columns[0]);

        // Colonna 2: selezione utenti collegati + chat singola (log e input)
        let col2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // selezione utenti collegati
                Constraint::Min(6),    // chat singola (log)
                Constraint::Length(5), // input chat singola
            ])
            .split(columns[1]);

        // Colonna 3: statistiche movimento (selezione utente/periodo + risultato)
        let col3 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // selezione utente / periodo
                Constraint::Min(6),    // stampa statistiche
            ])
            .split(columns[2]);

        self.draw_error_log(
            frame,
            root[1],
            matches!(self.focus, super::state::Panel::ErrorLog),
        );
        self.draw_help_bar(frame, root[2]);

        self.draw_users(
            frame,
            col1[0],
            matches!(self.focus, super::state::Panel::Users),
        );
        self.draw_broadcast(
            frame,
            col1[1],
            matches!(self.focus, super::state::Panel::BroadcastChat),
        );
        self.draw_broadcast_input(
            frame,
            col1[2],
            matches!(self.focus, super::state::Panel::Broadcast),
        );

        self.draw_connected_users(
            frame,
            col2[0],
            matches!(self.focus, super::state::Panel::SelectUser),
        );
        self.draw_direct_chat(
            frame,
            col2[1],
            matches!(self.focus, super::state::Panel::Chat),
        );
        self.draw_chat_input(
            frame,
            col2[2],
            matches!(self.focus, super::state::Panel::ChatInput),
        );

        self.draw_stats_select(
            frame,
            col3[0],
            matches!(self.focus, super::state::Panel::StatsSelect),
        );
        self.draw_stats(frame, col3[1]);
    }

    fn draw_users(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let total = self.users.len() as u16;
        let top_offset = Self::scroll_offset(total, area.height, self.users_scroll);
        let indicator = Self::scroll_indicator(total, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Utenti registrati{}", indicator))
            .border_style(border_style);

        let lines: Vec<ListItem> = self
            .users
            .iter()
            .skip(top_offset as usize)
            .map(|user| {
                let status_text = match user.status {
                    crate::state::UserStatus::Sconnesso => "[Sconnesso]",
                    crate::state::UserStatus::Fermo => "[Fermo]",
                    crate::state::UserStatus::InMovimento => "[In movimento]",
                    crate::state::UserStatus::Problema => "[Problema]",
                };
                let text = format!("{} {}", user.username, status_text);
                ListItem::new(text)
            })
            .collect();

        let list = List::new(lines).block(block);
        frame.render_widget(list, area);
    }

    fn draw_connected_users(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let unread_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

        let total = self.connected_users.connected_users.len() as u16;
        // Fa scorrere la lista in modo che l'utente selezionato resti sempre visibile:
        // riusa scroll_offset esprimendo la posizione selezionata come distanza dal fondo.
        let scroll_up = self
            .connected_users
            .index_selected
            .map(|idx| total.saturating_sub(1).saturating_sub(idx as u16))
            .unwrap_or(0);
        let top_offset = Self::scroll_offset(total, area.height, scroll_up);
        let indicator = Self::scroll_indicator(total, area.height, top_offset);

        let items = self
            .connected_users
            .connected_users
            .iter()
            .enumerate()
            .skip(top_offset as usize)
            .map(|(index, user)| {
                let label = if user.unread_count > 0 {
                    format!("{} ({})", user.username, user.unread_count)
                } else {
                    user.username.clone()
                };
                if Some(index) == self.connected_users.index_selected {
                    ListItem::new(label).style(style)
                } else if user.unread_count > 0 {
                    ListItem::new(label).style(unread_style)
                } else {
                    ListItem::new(label).style(Style::default())
                }
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Chat: seleziona utente{}", indicator))
                .border_style(border_style),
        );

        frame.render_widget(list, area);
    }

    pub fn draw_chat_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if self.connected_users.index_selected.is_none() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Scrivi messaggio")
                .border_style(border_style);
            let paragraph = Paragraph::new("Seleziona un utente collegato...")
                .block(block)
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, area);
            return;
        }

        let inner_width = area.width.saturating_sub(2);
        let visible_rows = area.height.saturating_sub(2);

        let total_lines = Self::wrapped_line_count(&self.chat_input, inner_width);
        let top_offset = Self::scroll_offset(total_lines, area.height, self.chat_input_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Scrivi messaggio{}", indicator))
            .border_style(border_style);

        let paragraph = Paragraph::new(self.chat_input.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);

        if focused {
            let (cursor_row, cursor_col) =
                Self::wrapped_cursor_position(&self.chat_input, inner_width);
            let screen_row = cursor_row as i32 - top_offset as i32;
            if screen_row >= 0 && screen_row < visible_rows as i32 {
                frame.set_cursor_position(Position::new(
                    area.x + 1 + cursor_col,
                    area.y + 1 + screen_row as u16,
                ));
            }
        }
    }

    fn draw_broadcast_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let inner_width = area.width.saturating_sub(2);
        let visible_rows = area.height.saturating_sub(2);

        let total_lines = Self::wrapped_line_count(&self.broadcast_chat_input, inner_width);
        let top_offset = Self::scroll_offset(total_lines, area.height, self.broadcast_input_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Scrivi messaggio Broadcast{}", indicator))
            .border_style(border_style);

        let paragraph = Paragraph::new(self.broadcast_chat_input.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);

        if focused {
            let (cursor_row, cursor_col) =
                Self::wrapped_cursor_position(&self.broadcast_chat_input, inner_width);
            let screen_row = cursor_row as i32 - top_offset as i32;
            if screen_row >= 0 && screen_row < visible_rows as i32 {
                frame.set_cursor_position(Position::new(
                    area.x + 1 + cursor_col,
                    area.y + 1 + screen_row as u16,
                ));
            }
        }
    }

    fn draw_broadcast(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let inner_width = area.width.saturating_sub(2);

        let formatted: Vec<String> = self
            .broadcast_log
            .iter()
            .map(|e| {
                let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                format!("[{}] > {}", time, e.text)
            })
            .collect();

        let total_lines: u16 = formatted
            .iter()
            .map(|text| Self::wrapped_line_count(text, inner_width))
            .sum();
        let top_offset = Self::scroll_offset(total_lines, area.height, self.broadcast_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Broadcast chat{}", indicator))
            .border_style(border_style);

        let lines: Vec<ratatui::text::Line> = formatted
            .into_iter()
            .map(|text| ratatui::text::Line::styled(text, Style::default()))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_error_log(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let inner_width = area.width.saturating_sub(2);

        let formatted: Vec<String> = self
            .error_log
            .iter()
            .map(|e| {
                let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                format!("[{}] {}", time, e.text)
            })
            .collect();

        let total_lines: u16 = formatted
            .iter()
            .map(|text| Self::wrapped_line_count(text, inner_width))
            .sum();
        let top_offset = Self::scroll_offset(total_lines, area.height, self.error_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Errori del server{}", indicator))
            .border_style(border_style);

        let lines: Vec<ratatui::text::Line> = formatted
            .into_iter()
            .map(|text| ratatui::text::Line::styled(text, Style::default().fg(Color::Red)))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_direct_chat(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let chat_with_user = if self.connected_users.index_selected.is_none() {
            "Nessun utente collegato selezionato".to_string()
        } else {
            format!(
                "Chat con: {}",
                self.connected_users.connected_users
                    [self.connected_users.index_selected.unwrap_or(0)]
                .username
            )
        };

        let user_selected_index = self.connected_users.index_selected;

        if user_selected_index.is_none() || self.connected_users.connected_users.is_empty() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(chat_with_user)
                .border_style(border_style);
            let paragraph = Paragraph::new("").block(block);
            frame.render_widget(paragraph, area);
            return;
        } else {
            let inner_width = area.width.saturating_sub(2);

            let select_chat_connected_user =
                self.connected_users.connected_users[user_selected_index.unwrap_or(0)].clone();

            let formatted: Vec<(String, Style)> = select_chat_connected_user
                .chat_log
                .iter()
                .map(|e| {
                    let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                    let text = if e.is_system {
                        format!("[{}] [sistema] {}", time, e.text)
                    } else if e.from_me {
                        format!("[{}] > {}", time, e.text)
                    } else {
                        format!("[{}] < {}", time, e.text)
                    };
                    let style = Style::default();
                    (text, style)
                })
                .collect();

            let total_lines: u16 = formatted
                .iter()
                .map(|(text, _)| Self::wrapped_line_count(text, inner_width))
                .sum();

            let top_offset = Self::scroll_offset(total_lines, area.height, self.chat_scroll);
            let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{}{}", chat_with_user, indicator))
                .border_style(border_style);

            let lines: Vec<ratatui::text::Line> = formatted
                .into_iter()
                .map(|(text, style)| ratatui::text::Line::styled(text, style))
                .collect();

            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((top_offset, 0));
            frame.render_widget(paragraph, area);
        }
    }

    fn draw_stats_select(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let highlight = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let (title, items): (&str, Vec<ListItem>) = match self.stats_step {
            StatsStep::SelectUser => {
                let items = self
                    .users
                    .iter()
                    .enumerate()
                    .map(|(i, u)| {
                        let item = ListItem::new(u.username.clone());
                        if Some(i) == self.stats_user_index {
                            item.style(highlight)
                        } else {
                            item
                        }
                    })
                    .collect();
                ("Statistiche: seleziona utente", items)
            }
            StatsStep::SelectPeriod => {
                let today = match self.stats_period {
                    TimePeriod::Today => highlight,
                    _ => Style::default(),
                };
                let this_week = match self.stats_period {
                    TimePeriod::ThisWeek => highlight,
                    _ => Style::default(),
                };
                let this_month = match self.stats_period {
                    TimePeriod::ThisMonth => highlight,
                    _ => Style::default(),
                };
                let items = vec![
                    ListItem::new("Oggi").style(today),
                    ListItem::new("Questa settimana").style(this_week),
                    ListItem::new("Questo mese").style(this_month),
                ];
                ("Statistiche: seleziona periodo", items)
            }
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        );

        frame.render_widget(list, area);
    }

    fn draw_stats(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Statistiche");

        let username = self.stats_username.as_deref().unwrap_or("-");

        let text = if let Some(err) = &self.stats_error {
            let time = self
                .stats_timestamp
                .map(|t| {
                    t.with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                        .to_string()
                })
                .unwrap_or_default();
            format!("[{}] Utente: {}\nErrore: {}", time, username, err)
        } else if let Some(stats) = &self.stats_result {
            format!(
                "Orario di stampa: {}\nUtente: {}\n\nDistanza: {:.2} km\nVelocità media: {:.2} km/h\nIn movimento: {}h {}min\nFermo: {}h {}min",
                self.stats_timestamp
                    .map(|t| t
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                        .to_string())
                    .unwrap_or_default(),
                username,
                stats.distance_km,
                stats.avg_speed_kmh,
                stats.moving_duration_secs / 3600,
                (stats.moving_duration_secs % 3600) / 60,
                stats.paused_duration_secs / 3600,
                (stats.paused_duration_secs % 3600) / 60,
            )
        } else {
            "Ancora nessuna richiesta".to_string()
        };

        frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: false }), area);
    }

    fn scroll_offset(total_lines: u16, area_height: u16, scroll_up: u16) -> u16 {
        let visible = area_height.saturating_sub(2); // meno le due righe di bordo
        let max_scroll = total_lines.saturating_sub(visible);
        let effective_scroll_up = scroll_up.min(max_scroll);
        max_scroll - effective_scroll_up
    }

    /// Suffisso da aggiungere al titolo di un riquadro per segnalare che il
    /// contenuto eccede lo spazio visibile e può essere scorso, con le stesse
    /// frecce usate nella barra di aiuto ("↑/↓").
    fn scroll_indicator(total_lines: u16, area_height: u16, top_offset: u16) -> &'static str {
        let visible = area_height.saturating_sub(2);
        let max_scroll = total_lines.saturating_sub(visible);
        if max_scroll == 0 {
            return "";
        }
        match (top_offset > 0, top_offset < max_scroll) {
            (true, true) => " ↑/↓",
            (true, false) => " ↑",
            (false, true) => " ↓",
            (false, false) => "",
        }
    }

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let hint = match self.focus {
            super::state::Panel::Users => "↑/↓: scorri la lista",
            super::state::Panel::SelectUser => "↑/↓: Selezione · Invio: Seleziona utente collegati",
            super::state::Panel::ChatInput | super::state::Panel::Broadcast => {
                "Digita il messaggio · ↑/↓: scorri se lungo · Invio: invia"
            }
            super::state::Panel::Chat | super::state::Panel::BroadcastChat => {
                "↑/↓: scorri lo storico"
            }
            super::state::Panel::StatsSelect => match self.stats_step {
                StatsStep::SelectUser => "↑/↓: seleziona utente · Invio: scegli periodo",
                StatsStep::SelectPeriod => {
                    "↑/↓: cambia periodo · Invio: interroga statistiche · Backspace: torna alla scelta utente"
                }
            },
            super::state::Panel::ErrorLog => "↑/↓: scorri il log errori",
        };
        let text = format!("Tab/Shift+Tab: cambia riquadro · {} · Esc: esci", hint);

        let block = Block::default().borders(Borders::ALL).title("Aiuto");
        frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: false }), area);
    }

    fn wrapped_line_count(text: &str, width: u16) -> u16 {
        if width == 0 {
            return 1;
        }
        let width = width as usize;
        text.split('\n')
            .map(|line| {
                let mut rows: u16 = 1;
                let mut current_len = 0;
                for word in line.split_whitespace() {
                    let word_len = word.chars().count();
                    if current_len == 0 {
                        current_len = word_len;
                    } else if current_len + 1 + word_len <= width {
                        current_len += 1 + word_len;
                    } else {
                        rows += 1;
                        current_len = word_len;
                    }
                }
                rows
            })
            .sum()
    }

    fn wrapped_cursor_position(text: &str, width: u16) -> (u16, u16) {
        if width == 0 {
            return (0, 0);
        }
        let width = width as usize;
        let mut row: u16 = 0;
        let mut current_len = 0usize;
        for word in text.split_whitespace() {
            let word_len = word.chars().count();
            if current_len == 0 {
                current_len = word_len;
            } else if current_len + 1 + word_len <= width {
                current_len += 1 + word_len;
            } else {
                row += 1;
                current_len = word_len;
            }
        }
        (row, current_len as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- wrapped_line_count ---

    #[test]
    fn testo_vuoto_conta_una_riga() {
        assert_eq!(App::wrapped_line_count("", 20), 1);
    }

    #[test]
    fn larghezza_zero_conta_una_riga() {
        assert_eq!(App::wrapped_line_count("qualsiasi cosa", 0), 1);
    }

    #[test]
    fn testo_corto_sta_in_una_riga() {
        assert_eq!(App::wrapped_line_count("ciao mondo", 20), 1);
    }

    #[test]
    fn testo_lungo_va_a_capo() {
        // "ciao" (4) + spazio + "mondo" (5) = 10 > larghezza 8, quindi 2 righe
        assert_eq!(App::wrapped_line_count("ciao mondo", 8), 2);
    }

    #[test]
    fn a_capo_esplicito_viene_rispettato() {
        assert_eq!(App::wrapped_line_count("prima\nseconda", 20), 2);
    }

    #[test]
    fn testo_molto_lungo_va_a_capo_piu_volte() {
        assert_eq!(App::wrapped_line_count("uno due tre quattro", 4), 4);
    }

    // --- wrapped_cursor_position ---

    #[test]
    fn cursore_su_testo_vuoto_e_in_origine() {
        assert_eq!(App::wrapped_cursor_position("", 20), (0, 0));
    }

    #[test]
    fn cursore_su_testo_corto_resta_sulla_prima_riga() {
        assert_eq!(App::wrapped_cursor_position("ciao mondo", 20), (0, 10));
    }

    #[test]
    fn cursore_dopo_un_a_capo_e_sulla_seconda_riga() {
        // "ciao" (4) + spazio + "mondo" (5) = 10 > larghezza 8: "mondo" va a
        // capo, quindi il cursore finisce a riga 1, colonna 5 (lunghezza di "mondo")
        assert_eq!(App::wrapped_cursor_position("ciao mondo", 8), (1, 5));
    }

    // --- scroll_offset ---

    #[test]
    fn contenuto_che_ci_sta_tutto_non_scorre() {
        // 3 righe di contenuto, 5 visibili: nessuno scroll necessario
        assert_eq!(App::scroll_offset(3, 7, 0), 0);
        assert_eq!(App::scroll_offset(3, 7, 10), 0);
    }

    #[test]
    fn senza_scroll_manuale_si_vede_il_fondo() {
        // 10 righe di contenuto, 5 visibili (area_height 7 - 2 di bordo):
        // con scroll_up=0 l'offset deve mostrare le ultime 5 righe
        assert_eq!(App::scroll_offset(10, 7, 0), 5);
    }

    #[test]
    fn scroll_manuale_riduce_l_offset() {
        assert_eq!(App::scroll_offset(10, 7, 3), 2);
    }

    #[test]
    fn scroll_oltre_il_massimo_si_ferma_all_inizio() {
        assert_eq!(App::scroll_offset(10, 7, 5), 0);
        assert_eq!(App::scroll_offset(10, 7, 100), 0);
    }
}
