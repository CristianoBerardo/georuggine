use crate::auth::verify_password;
use crate::db::{delete_user, get_user_by_username};
use crate::state::AppState;
use crate::user_status::remove_user_from_status_map;

use common::protocol::ServerMessage;
use tokio::net::tcp::OwnedWriteHalf;

use crate::messaging::send_message;

pub async fn handle_delete_account(
    password: String,
    state: &AppState,
    writer: &mut OwnedWriteHalf,
    authenticated_user: &mut Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(username) = authenticated_user.clone() else {
        let err = ServerMessage::AccountDeleted {
            success: false,
            reason: Some("Devi essere autenticato per eliminare l'account.".to_string()),
            timestamp: chrono::Utc::now(),
        };
        send_message(writer, &err).await?;
        return Ok(());
    };

    match get_user_by_username(&state.db, &username).await {
        Ok(Some(user)) if verify_password(&user.password_hash, &password) => {
            let user_id = user
                .id
                .expect("un utente autenticato deve avere un id valido nel DB");
            delete_user(&state.db, user_id).await?;

            {
                let mut conns = state.connections.write().await;
                conns.remove(&username);
            }
            remove_user_from_status_map(state, &username).await?;
            let _ = state.connections_notify.send(());

            let ok = ServerMessage::AccountDeleted {
                success: true,
                reason: None,
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &ok).await?;

            // Non c'è più un utente autenticato su questa connessione: il client
            // si disconnetterà da solo per tornare alla schermata di login.
            *authenticated_user = None;
        }
        Ok(_) => {
            let err = ServerMessage::AccountDeleted {
                success: false,
                reason: Some("Password errata.".to_string()),
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &err).await?;
        }
        Err(e) => {
            let message = format!("Errore DB durante l'eliminazione di {}: {}", username, e);
            let _ = state.error_tx.send(crate::state::IncomingError {
                message,
                timestamp: chrono::Utc::now(),
            });
            let err = ServerMessage::AccountDeleted {
                success: false,
                reason: Some(format!("Errore interno del server: {}", e)),
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &err).await?;
        }
    }
    Ok(())
}
