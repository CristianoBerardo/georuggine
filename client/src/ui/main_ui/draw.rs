use super::state::App;
use common::protocol::TimePeriod;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        // root[0] AREA PRINCIPALE
        // root[1] AREA DI AIUTO
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(frame.area());

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(root[0]);

        let col1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // info utente
                Constraint::Length(6), // info movimento
                Constraint::Length(5), // scelta periodo statistiche
                Constraint::Min(6),    // stampa statistiche
            ])
            .split(columns[0]);

        let col2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // messaggi broadcast
                Constraint::Min(10),   // chat diretta
                Constraint::Length(3), // input messaggi
            ])
            .split(columns[1]);

        self.draw_user_info(
            frame,
            col1[0],
            matches!(self.focus, super::state::Panel::UserInfo),
        );
        self.draw_movement(
            frame,
            col1[1],
            matches!(self.focus, super::state::Panel::Movement),
        );
        self.draw_stats_period(
            frame,
            col1[2],
            matches!(self.focus, super::state::Panel::StatsPeriod),
        );
        self.draw_stats(
            frame,
            col1[3],
            matches!(self.focus, super::state::Panel::Stats),
        );
        self.draw_broadcast(
            frame,
            col2[0],
            matches!(self.focus, super::state::Panel::Broadcast),
        );
        self.draw_chat(
            frame,
            col2[1],
            matches!(self.focus, super::state::Panel::Chat),
        );
        self.draw_chat_input(
            frame,
            col2[2],
            matches!(self.focus, super::state::Panel::ChatInput),
        );
        self.draw_help_bar(frame, root[1]);
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

        let paragraph = Paragraph::new(self.username.as_str()).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_chat(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Chat")
            .border_style(border_style);

        let lines: Vec<ratatui::text::Line> = self
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
                let style = if e.is_system {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };
                ratatui::text::Line::styled(text, style)
            })
            .collect();

        let top_offset =
            Self::scroll_offset(self.chat_log.len() as u16, area.height, self.chat_scroll);

        let paragraph = Paragraph::new(lines).block(block).scroll((top_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_chat_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Scrivi messaggio")
            .border_style(border_style);
        let paragraph = Paragraph::new(self.chat_input.as_str()).block(block);
        frame.render_widget(paragraph, area);

        if focused {
            frame.set_cursor_position(Position::new(
                area.x + self.chat_input.chars().count() as u16 + 1,
                area.y + 1,
            ));
        }
    }

    fn draw_broadcast(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Broadcast")
            .border_style(border_style);
        let text: Vec<String> = self
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

        let top_offset = Self::scroll_offset(
            self.broadcast_log.len() as u16,
            area.height,
            self.broadcast_scroll,
        );
        let paragraph = Paragraph::new(text.join("\n"))
            .block(block)
            .scroll((top_offset, 0));

        frame.render_widget(paragraph, area);
    }

    fn scroll_offset(total_lines: u16, area_height: u16, scroll_up: u16) -> u16 {
        let visible = area_height.saturating_sub(2); // meno le due righe di bordo
        let max_scroll = total_lines.saturating_sub(visible);
        let effective_scroll_up = scroll_up.min(max_scroll);
        max_scroll - effective_scroll_up
    }

    fn draw_stats_period(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let today = match self.selected_period {
            TimePeriod::Today => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };

        let this_week = match self.selected_period {
            TimePeriod::ThisWeek => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };

        let this_month = match self.selected_period {
            TimePeriod::ThisMonth => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };

        let items = vec![
            ListItem::new("Oggi").style(today),
            ListItem::new("Questa settimana").style(this_week),
            ListItem::new("Questo mese").style(this_month),
        ];

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Periodo statistiche")
                .border_style(border_style),
        );

        frame.render_widget(list, area);
    }

    fn draw_stats(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Statistiche")
            .border_style(border_style);

        let text = if self.stats_pending {
            "In attesa della risposta dal server...".to_string()
        } else if let Some(err) = &self.stats_error {
            let time = self
                .stats_timestamp
                .map(|t| {
                    t.with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                        .to_string()
                })
                .unwrap_or_default();
            format!("[{}] Errore: {}", time, err)
        } else if let Some(stats) = &self.stats {
            format!(
                "Orario di stampa: {}\n\nDistanza: {:.2} km\nVelocità media: {:.2} km/h\nIn movimento: {}h {}min\nFermo: {}h {}min",
                self.stats_timestamp
                    .map(|t| t
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                        .to_string())
                    .unwrap_or_default(),
                stats.distance_km,
                stats.avg_speed_kmh,
                stats.moving_duration_secs / 3600,
                (stats.moving_duration_secs % 3600) / 60,
                stats.paused_duration_secs / 3600,
                (stats.paused_duration_secs % 3600) / 60,
            )
        } else {
            "Nessuna richiesta ancora (Invio sul periodo per interrogare)".to_string()
        };

        frame.render_widget(Paragraph::new(text).block(block), area);
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
            Some(pos) if !self.movement_status.finished => (
                format!(
                    "Stato: In movimento\nPrimo invio: [{}]\nPosizione: {:.5}, {:.5}\nUltimo invio: [{}]",
                    first_sent,
                    pos.lat,
                    pos.lon,
                    pos.timestamp
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                ),
                Style::default(),
            ),
            Some(pos) => (
                format!(
                    "Stato: Sconnesso\nPrimo invio: [{}]\nUltima posizione: {:.5}, {:.5}\nUltimo invio: [{}]",
                    first_sent,
                    pos.lat,
                    pos.lon,
                    pos.timestamp
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                ),
                Style::default().fg(Color::Red),
            ),
        };

        frame.render_widget(Paragraph::new(text).style(text_style).block(block), area);
    }

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let hint = match self.focus {
            super::state::Panel::StatsPeriod => {
                "↑/↓: cambia periodo · Invio: interroga statistiche"
            }
            super::state::Panel::ChatInput => "Digita il messaggio · Invio: invia",
            super::state::Panel::Chat | super::state::Panel::Broadcast => "↑/↓: scorri lo storico",
            _ => "Sola lettura",
        };
        let text = format!("Tab/Backtab: cambia riquadro · {} · Esc: esci", hint);

        let block = Block::default().borders(Borders::ALL).title("Aiuto");
        frame.render_widget(Paragraph::new(text).block(block), area);
    }
}
