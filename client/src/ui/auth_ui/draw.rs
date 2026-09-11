use common::protocol::AuthAction;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use super::state::{App, Focus, Screen};
use crate::ui::size_control::size_too_small;

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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([
                Constraint::Length(4), // 2 voci + 2 righe di bordo
                Constraint::Length(1), // suggerimento
                Constraint::Min(0),    // spazio vuoto restante
            ])
            .split(frame.area());

        frame.render_widget(list, chunks[0]);
        frame.render_widget(
            Paragraph::new("Esc per uscire dall'applicazione"),
            chunks[1],
        );
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
                Constraint::Length(1), // suggerimento
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
        frame.render_widget(Paragraph::new("Esc per tornare indietro"), chunks[3]);
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
                Constraint::Length(1), // suggerimento
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
        frame.render_widget(Paragraph::new("Esc per tornare indietro"), chunks[4]);
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

        let paragraph = Paragraph::new(text)
            .style(style)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}
