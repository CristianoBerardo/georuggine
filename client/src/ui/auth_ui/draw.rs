use common::protocol::AuthAction;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::state::{App, Focus, Screen};

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        match self.screen {
            Screen::ChooseAction => self.draw_choose_action(frame),
            Screen::Login => self.draw_login(frame),
            Screen::Register => self.draw_register(frame),
        }
    }

    fn draw_choose_action(&self, frame: &mut Frame) {
        // Disegna il menu di scelta tra login e registrazione
        let login_style = match self.selected_action {
            AuthAction::Login => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };

        let register_style = match self.selected_action {
            AuthAction::Register => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };

        let items = vec![
            ListItem::new("Login").style(login_style),
            ListItem::new("Registrazione").style(register_style),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Scegli un'azione"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let area = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Min(1)])
            .split(frame.area())[0];

        frame.render_widget(list, area);
    }

    fn draw_login(&self, frame: &mut Frame) {
        // Disegna il form di login
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(frame.area());

        self.draw_field(
            frame,
            chunks[0],
            "Username",
            &self.username,
            false,
            matches!(self.focus, Focus::Username),
        );
        self.draw_field(
            frame,
            chunks[1],
            "Password",
            &self.password,
            true,
            matches!(self.focus, Focus::Password),
        );
        self.draw_status(frame, chunks[2]);
    }

    fn draw_register(&self, frame: &mut Frame) {
        // Disegna il form di registrazione
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
            ])
            .split(frame.area());

        self.draw_field(
            frame,
            chunks[0],
            "Username",
            &self.username,
            false,
            matches!(self.focus, Focus::Username),
        );
        self.draw_field(
            frame,
            chunks[1],
            "Password",
            &self.password,
            true,
            matches!(self.focus, Focus::Password),
        );
        self.draw_field(
            frame,
            chunks[2],
            "Conferma Password",
            &self.confirm_password,
            true,
            matches!(self.focus, Focus::ConfirmPassword),
        );
        self.draw_status(frame, chunks[3]);
    }

    fn draw_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        value: &str,
        masked: bool,
        focused: bool,
    ) {
        let display: String = if masked {
            "*".repeat(value.chars().count())
        } else {
            value.to_string()
        };

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(border_style);

        let paragraph = Paragraph::new(display.as_str()).block(block);
        frame.render_widget(paragraph, area);

        if focused {
            frame.set_cursor_position(Position::new(
                area.x + display.chars().count() as u16 + 1,
                area.y + 1,
            ));
        }
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let (text, style) = if self.awaiting_response {
            (
                "In attesa di risposta dal server...".to_string(),
                Style::default().fg(Color::Cyan),
            )
        } else if let Some(err) = &self.error_message {
            (err.clone(), Style::default().fg(Color::Red))
        } else if let Some(msg) = &self.info_message {
            (msg.clone(), Style::default().fg(Color::Green))
        } else {
            (String::new(), Style::default())
        };

        let paragraph = Paragraph::new(text).style(style);
        frame.render_widget(paragraph, area);
    }
}
