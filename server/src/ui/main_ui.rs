use crate::ui::terminal_guard::TerminalGuard;

mod draw;
mod state;

pub async fn run(
    username: String,
    // client_msg_tx: &Sender<ClientMessage>,
    // server_msg_rx: &mut Receiver<ServerMessage>,
    // movement_status_rx: &mut watch::Receiver<MovementStatus>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = state::App::new(username);

    // Inizializza il terminale
    let mut terminal = ratatui::init();
    let _terminal_guard = TerminalGuard;

    loop {
        terminal.draw(|frame| app.draw_placeholder(frame, frame.size(), "Test", false))?;
    }

    Ok(())
}
