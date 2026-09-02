use super::state::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

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
        self.draw_placeholder(
            frame,
            col1[2],
            "Periodo statistiche",
            matches!(self.focus, super::state::Panel::StatsPeriod),
        );
        self.draw_placeholder(
            frame,
            col1[3],
            "Statistiche",
            matches!(self.focus, super::state::Panel::Stats),
        );
        self.draw_placeholder(
            frame,
            col2[0],
            "Broadcast",
            matches!(self.focus, super::state::Panel::Broadcast),
        );
        self.draw_placeholder(
            frame,
            col2[1],
            "Chat",
            matches!(self.focus, super::state::Panel::Chat),
        );
        self.draw_placeholder(
            frame,
            col2[2],
            "Scrivi messaggio",
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
}
