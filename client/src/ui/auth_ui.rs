mod draw;
mod input;
mod state;

use crate::ui::terminal_guard::TerminalGuard;

use common::protocol::{ClientMessage, ServerMessage};
use futures::StreamExt;
use input::Outbound;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use state::{App, Focus, Screen};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn run(
    tx: &Sender<ClientMessage>,
    auth_resp_rx: &mut Receiver<ServerMessage>,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
                                    return Ok(None);
                                }
                            }
                            Outbound::SendRegister { username, password } => {
                                let msg = ClientMessage::Register { username, password };
                                if tx.send(msg).await.is_err() {
                                    return Ok(None);
                                }
                            }
                            Outbound::Cancel => {
                                return Ok(None);
                            }
                            Outbound::None => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return Ok(None),
                }
            }
            maybe_msg = auth_resp_rx.recv() => {
                match maybe_msg {
                    Some(ServerMessage::AuthResult { success: true, .. }) => {
                        match app.screen {
                            Screen::Login => {
                                return Ok(Some(app.username.clone()));
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
                    Some(ServerMessage::AuthResult { success: false, reason, .. }) => {
                        app.note_failure(reason.unwrap_or_else(|| "Autenticazione fallita.".to_string()));
                    }
                    Some(ServerMessage::Error { message, .. }) => {
                        app.note_failure(message);
                    }
                    Some(_) => {}
                    None => return Ok(None),
                }
            }
        }
    }
}
