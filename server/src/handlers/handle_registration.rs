use crate::auth::hash_password;
use crate::db::insert_user;
use crate::menu;
use crate::state::AppState;
use crate::user_status::add_user_to_status_map;

use common::protocol::ServerMessage;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;

use crate::messaging::send_message;

pub async fn handle_registration(
    username: String,
    password: String,
    state: &AppState,
    writer: &mut OwnedWriteHalf,
    tx: &UnboundedSender<ServerMessage>,
    authenticated_user: &mut Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    menu::print_or_queue_with_menu(format!(
        "Tentativo di registrazione per l'utente: {}",
        username
    ));
    let password_hash = hash_password(&password)?;
    match insert_user(&state.db, username.clone(), password_hash).await {
        Ok(_) => {
            menu::print_or_queue_with_menu(format!("Utente {} registrato con successo!", username));
            *authenticated_user = Some(username.clone());

            // Registra il canale nella mappa delle connessioni
            {
                let mut conns = state.connections.write().await;
                conns.insert(username.clone(), tx.clone());
            }

            // Invia conferma di registrazione
            let reg_ok = ServerMessage::AuthResult {
                success: true,
                reason: None,
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &reg_ok).await?;
            add_user_to_status_map(state, username).await?;
        }
        Err(e) => {
            eprintln!("Errore durante la registrazione di {}: {}", username, e);
            let reg_err = ServerMessage::AuthResult {
                success: false,
                reason: Some(format!("Errore durante la registrazione: {}", e)),
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &reg_err).await?;
        }
    }
    Ok(())
}
