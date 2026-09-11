use super::state::App;
use crate::movement_sim::MovementState;
use crate::ui::main_ui::state::{DeleteAccountStep, Panel};
use crate::ui::size_control::size_too_small;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(root[0]);

        let col1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // nome utente
                Constraint::Length(3), // elimina account
                Constraint::Length(6), // stato movimento
                Constraint::Min(5),    // messaggi broadcast
            ])
            .split(columns[0]);

        let col2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // chat con il server
                Constraint::Length(4), // input messaggi
            ])
            .split(columns[1]);

        self.draw_user_info(
            frame,
            col1[0],
            matches!(self.focus, super::state::Panel::UserInfo),
        );
        self.draw_delete_account(
            frame,
            col1[1],
            matches!(self.focus, super::state::Panel::DeleteAccount),
        );
        self.draw_movement(
            frame,
            col1[2],
            matches!(self.focus, super::state::Panel::Movement),
        );
        self.draw_broadcast(
            frame,
            col1[3],
            matches!(self.focus, super::state::Panel::Broadcast),
        );
        self.draw_chat(
            frame,
            col2[0],
            matches!(self.focus, super::state::Panel::Chat),
        );
        self.draw_chat_input(
            frame,
            col2[1],
            matches!(self.focus, super::state::Panel::ChatInput),
        );
        self.draw_error_log(
            frame,
            root[1],
            matches!(self.focus, super::state::Panel::ErrorLog),
        );
        self.draw_help_bar(frame, root[2]);
    }

    fn draw_user_info(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Utente")
            .border_style(border_style);

        let paragraph = Paragraph::new(self.username.as_str())
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn draw_chat(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let inner_width = area.width.saturating_sub(2);

        let formatted: Vec<String> = self
            .chat_log
            .iter()
            .map(|e| {
                let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                if e.from_me {
                    format!("[{}] > {}", time, e.text)
                } else {
                    format!("[{}] < {}", time, e.text)
                }
            })
            .collect();

        let total_lines: u16 = formatted
            .iter()
            .map(|text| Self::wrapped_line_count(text, inner_width))
            .sum();

        let top_offset = Self::scroll_offset(total_lines, area.height, self.chat_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Chat{}", indicator))
            .border_style(border_style);

        let paragraph = Paragraph::new(formatted.join("\n"))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_chat_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
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

    fn draw_broadcast(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let inner_width = area.width.saturating_sub(2);

        let entries: Vec<String> = self
            .broadcast_log
            .iter()
            .map(|entry| {
                format!(
                    "[{}] {}",
                    entry
                        .timestamp
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S"),
                    entry.text
                )
            })
            .collect();

        let total_lines: u16 = entries
            .iter()
            .map(|text| Self::wrapped_line_count(text, inner_width))
            .sum();

        let top_offset = Self::scroll_offset(total_lines, area.height, self.broadcast_scroll);
        let indicator = Self::scroll_indicator(total_lines, area.height, top_offset);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Broadcast{}", indicator))
            .border_style(border_style);

        let paragraph = Paragraph::new(entries.join("\n"))
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
            .title(format!("Errori{}", indicator))
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

    fn scroll_offset(total_lines: u16, area_height: u16, scroll_up: u16) -> u16 {
        let visible = area_height.saturating_sub(2); // meno le due righe di bordo
        let max_scroll = total_lines.saturating_sub(visible);
        let effective_scroll_up = scroll_up.min(max_scroll);
        max_scroll - effective_scroll_up
    }

    /// Suffisso da aggiungere al titolo di un riquadro per segnalare che il
    /// contenuto eccede lo spazio visibile e può essere scorso, con le stesse
    /// frecce usate nella barra di aiuto ("↑/↓")
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

    fn draw_movement(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Stato movimento")
            .border_style(border_style);

        let first_sent = self
            .movement_status
            .first_sent_position
            .as_ref()
            .map(|pos| {
                pos.timestamp
                    .with_timezone(&chrono::Local)
                    .format("%H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default();

        let (text, text_style) = match &self.movement_status.last_sent_position {
            None => ("In attesa del primo invio...".to_string(), Style::default()),
            Some(pos) => {
                let last_sent = pos
                    .timestamp
                    .with_timezone(&chrono::Local)
                    .format("%H:%M:%S");

                let (label, style) = match self.movement_status.state {
                    MovementState::InMovimento => ("In movimento", Style::default()),
                    MovementState::Fermo => ("Fermo", Style::default().fg(Color::Yellow)),
                    MovementState::Problema => ("Problema", Style::default().fg(Color::Red)),
                };

                (
                    format!(
                        "Stato: {}\nPrimo invio: [{}]\nPosizione: {:.5}, {:.5}\nUltimo invio: [{}]",
                        label, first_sent, pos.lat, pos.lon, last_sent
                    ),
                    style,
                )
            }
        };

        frame.render_widget(
            Paragraph::new(text)
                .style(text_style)
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn draw_delete_account(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Elimina account")
            .border_style(border_style);

        let mut password_prefix_len = 0;

        let text = if self.delete_pending {
            "Eliminazione in corso...".to_string()
        } else {
            match self.delete_step {
                DeleteAccountStep::Idle => self
                    .delete_error
                    .clone()
                    .unwrap_or_else(|| "Invio per eliminare l'account".to_string()),
                DeleteAccountStep::EnterPassword => {
                    let prefix = "Password: ";
                    password_prefix_len = prefix.chars().count();
                    let masked = "*".repeat(self.delete_password.chars().count());
                    match &self.delete_error {
                        Some(err) => format!("{}{}\n{}", prefix, masked, err),
                        None => format!("{}{}", prefix, masked),
                    }
                }
                DeleteAccountStep::Confirm => {
                    "Eliminare DAVVERO l'account? Azione irreversibile. (y/n)".to_string()
                }
            }
        };

        let style = if self.delete_pending {
            Style::default().fg(Color::Cyan)
        } else {
            match self.delete_step {
                DeleteAccountStep::Idle if self.delete_error.is_some() => {
                    Style::default().fg(Color::Red)
                }
                DeleteAccountStep::Idle => Style::default(),
                DeleteAccountStep::EnterPassword => Style::default(),
                DeleteAccountStep::Confirm => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }
            }
        };

        frame.render_widget(
            Paragraph::new(text)
                .style(style)
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );

        if focused && !self.delete_pending && self.delete_step == DeleteAccountStep::EnterPassword {
            let cursor_col = password_prefix_len + self.delete_password.chars().count();
            frame.set_cursor_position(Position::new(area.x + 1 + cursor_col as u16, area.y + 1));
        }
    }

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let hint = match self.focus {
            Panel::ChatInput => "Digita il messaggio · ↑/↓: scorri se lungo · Invio: invia",
            Panel::Chat | Panel::Broadcast | Panel::ErrorLog => "↑/↓: scorri lo storico",
            Panel::DeleteAccount => "Invio: elimina account (richiede password e conferma)",
            _ => "Sola lettura",
        };
        let text = format!("Tab/Shift+Tab: cambia riquadro · {} · Esc: esci", hint);

        let block = Block::default().borders(Borders::ALL).title("Aiuto");
        frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: false }), area);
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
