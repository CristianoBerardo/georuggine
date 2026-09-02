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
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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
        self.draw_placeholder(
            frame,
            col1[1],
            "Stato movimento",
            matches!(self.focus, super::state::Panel::Movement),
        );
        self.draw_stats_period(
            frame,
            col1[2],
            matches!(self.focus, super::state::Panel::StatsPeriod),
        );
        self.draw_placeholder(
            frame,
            col1[3],
            "Statistiche",
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
        self.draw_placeholder(frame, root[1], "Premi ESC per uscire", false);
    }

    fn draw_placeholder(&self, frame: &mut Frame, area: Rect, title: &str, focused: bool) {
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
        let lines: Vec<String> = self
            .chat_log
            .iter()
            .map(|e| {
                if e.from_me {
                    format!("> {}", e.text)
                } else {
                    format!("< {}", e.text)
                }
            })
            .collect();
        let paragraph = Paragraph::new(lines.join("\n")).block(block);
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
        let paragraph = Paragraph::new(self.broadcast_log.join("\n")).block(block);
        frame.render_widget(paragraph, area);
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
}
