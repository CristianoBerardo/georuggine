use crate::ui::auth_ui;
use common::protocol::{ClientMessage, ServerMessage};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn authenticate(
    tx: &Sender<ClientMessage>,
    rx: &mut Receiver<ServerMessage>,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    auth_ui::run(tx, rx).await
}
