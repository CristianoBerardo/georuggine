use common::protocol::{AuthAction, ClientMessage, ServerMessage};
use futures::StreamExt;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tokio::sync::mpsc::{Receiver, Sender};

enum Screen {
    ChooseAction,
    Login,
    Register,
}

enum Focus {
    Username,
    Password,
    ConfirmPassword,
}

enum Outbound {
    SendLogin { username: String, password: String },
    SendRegister { username: String, password: String },
    Cancel,
    None,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

struct App {
    screen: Screen,
    focus: Focus,
    selected_action: AuthAction,
    username: String,
    password: String,
    confirm_password: String,
    error_message: Option<String>,
    info_message: Option<String>,
    awaiting_response: bool,
}

impl App {
    fn new() -> Self {
        App {
            screen: Screen::ChooseAction,
            focus: Focus::Username,
            selected_action: AuthAction::Login,
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            error_message: None,
            info_message: None,
            awaiting_response: false,
        }
    }

    fn draw(&self, frame: &mut Frame) {
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
            .margin(20)
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

    fn handle_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente in base allo schermo attuale e al focus
        if self.awaiting_response {
            if key.code == KeyCode::Esc {
                self.awaiting_response = false;
                self.error_message = None;
                self.info_message = None;
                return Outbound::Cancel;
            } else {
                return Outbound::None;
            }
        }

        match self.screen {
            Screen::ChooseAction => self.handle_selected_action_key(key),
            Screen::Login => self.handle_login_key(key),
            Screen::Register => self.handle_register_key(key),
        }
    }

    fn handle_selected_action_key(&mut self, key: KeyEvent) -> Outbound {
        if key.code == KeyCode::Up || key.code == KeyCode::Down {
            // Toggle dell'azione selezionata tra login e registrazione
            self.selected_action = match self.selected_action {
                AuthAction::Login => AuthAction::Register,
                AuthAction::Register => AuthAction::Login,
            };
        } else if key.code == KeyCode::Enter {
            self.screen = match self.selected_action {
                AuthAction::Login => {
                    self.focus = Focus::Username;
                    self.error_message = None;
                    self.info_message = None;
                    Screen::Login
                }
                AuthAction::Register => {
                    self.focus = Focus::Username;
                    self.error_message = None;
                    self.info_message = None;
                    Screen::Register
                }
            };
        } else if key.code == KeyCode::Esc {
            self.awaiting_response = false;
            // Gestisci l'uscita dal menu di scelta
            return Outbound::Cancel;
        }
        Outbound::None
    }

    fn handle_login_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente nel form di login
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Username => Focus::Password,
                    Focus::Password => Focus::Username,
                    _ => Focus::Username,
                }
            }
            KeyCode::Char(c) => match self.focus {
                Focus::Username => self.username.push(c),
                Focus::Password => self.password.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                Focus::Username => {
                    self.username.pop();
                }
                Focus::Password => {
                    self.password.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error_message =
                        Some("Username e password non possono essere vuoti.".to_string());
                    return Outbound::None;
                } else {
                    self.error_message = None;
                    self.awaiting_response = true;
                    return Outbound::SendLogin {
                        username: self.username.clone(),
                        password: self.password.clone(),
                    };
                }
            }
            KeyCode::Esc => {
                return Outbound::Cancel;
            }
            _ => {
                return Outbound::None;
            }
        }
        Outbound::None
    }

    fn handle_register_key(&mut self, key: KeyEvent) -> Outbound {
        // Gestisci l'input dell'utente nel form di registrazione
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Username => Focus::Password,
                    Focus::Password => Focus::ConfirmPassword,
                    Focus::ConfirmPassword => Focus::Username,
                    _ => Focus::Username,
                }
            }
            KeyCode::Char(c) => match self.focus {
                Focus::Username => self.username.push(c),
                Focus::Password => self.password.push(c),
                Focus::ConfirmPassword => self.confirm_password.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                Focus::Username => {
                    self.username.pop();
                }
                Focus::Password => {
                    self.password.pop();
                }
                Focus::ConfirmPassword => {
                    self.confirm_password.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                if self.username.is_empty()
                    || self.password.is_empty()
                    || self.confirm_password.is_empty()
                {
                    self.error_message =
                        Some("Username e password non possono essere vuoti.".to_string());
                    return Outbound::None;
                } else if self.password != self.confirm_password {
                    self.error_message = Some("Le password non corrispondono.".to_string());
                    return Outbound::None;
                } else {
                    self.error_message = None;
                    self.awaiting_response = true;
                    return Outbound::SendRegister {
                        username: self.username.clone(),
                        password: self.password.clone(),
                    };
                }
            }
            KeyCode::Esc => {
                return Outbound::Cancel;
            }
            _ => {
                return Outbound::None;
            }
        }
        Outbound::None
    }

    fn note_failure(&mut self, message: String) {
        self.awaiting_response = false;
        self.error_message = Some(message);
    }
}

pub async fn run(
    tx: &Sender<ClientMessage>,
    auth_resp_rx: &mut Receiver<ServerMessage>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard;
    let mut term_events = EventStream::new();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        tokio::select! {
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        match app.handle_key(key) {
                            Outbound::SendLogin { username, password } => {
                                let msg = ClientMessage::Login { username, password };
                                if tx.send(msg).await.is_err() {
                                    return Ok(false);
                                }
                            }
                            Outbound::SendRegister { username, password } => {
                                let msg = ClientMessage::Register { username, password };
                                if tx.send(msg).await.is_err() {
                                    return Ok(false);
                                }
                            }
                            Outbound::Cancel => {
                                return Ok(false);
                            }
                            Outbound::None => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(false),
                }
            }
            maybe_msg = auth_resp_rx.recv() => {
                match maybe_msg {
                    Some(ServerMessage::AuthResult { success: true, reason }) => {
                        match app.screen {
                            Screen::Login => {
                                return Ok(true);
                            }
                            Screen::Register => {
                                app.info_message = Some("Registrazione completata con successo!".to_string());
                                app.username = String::new();
                                app.password = String::new();
                                app.confirm_password = String::new();
                                app.awaiting_response = false;
                                app.error_message = None;
                                app.focus = Focus::Username;
                                app.screen = Screen::Login;
                            }
                            _ => {}
                        }
                    }
                    Some(ServerMessage::AuthResult { success: false, reason }) => {
                        app.note_failure(reason.unwrap_or_else(|| "Autenticazione fallita.".to_string()));
                    }
                    Some(ServerMessage::Error { message }) => {
                        app.note_failure(message);
                    }
                    Some(_) => {}
                    None => return Ok(false),
                }
            }
        }
    }
}
