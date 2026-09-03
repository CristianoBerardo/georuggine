#[allow(dead_code)]
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

        self.draw_chat_input(
            frame,
            col2[2],
            matches!(self.focus, super::state::Panel::ChatInput),
        );
        self.draw_help_bar(frame, root[1]);
    }

    pub(crate) fn draw_placeholder(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        focused: bool,
    ) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);
        frame.render_widget(block, area);
    }

    pub fn draw_chat_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
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

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let hint = match self.focus {
            super::state::Panel::SelectUser => "↑/↓: Selezione · Invio: Seleziona utente",
            super::state::Panel::ChatInput => "Digita il messaggio · Invio: invia",
            super::state::Panel::Chat | super::state::Panel::Broadcast => "↑/↓: scorri lo storico",
            _ => "Sola lettura",
        };
        let text = format!("Tab/Backtab: cambia riquadro · {} · Esc: esci", hint);

        let block = Block::default().borders(Borders::ALL).title("Aiuto");
        frame.render_widget(Paragraph::new(text).block(block), area);
    }
}
