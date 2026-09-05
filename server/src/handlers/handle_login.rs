use crate::auth::verify_password;
use crate::db::get_user_by_username;
use crate::state::AppState;

use common::protocol::{ErrorContext, ServerMessage};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;

use crate::messaging::send_message;

pub async fn handle_login(
    username: String,
    password: String,
    state: &AppState,
    writer: &mut OwnedWriteHalf,
    tx: &UnboundedSender<ServerMessage>,
    authenticated_user: &mut Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match get_user_by_username(&state.db, &username).await {
        Ok(Some(user)) if verify_password(&user.password_hash, &password) => {
            *authenticated_user = Some(username.clone());

            // Registra il canale nella mappa delle connessioni
            {
                let mut conns = state.connections.write().await;
                conns.insert(username.clone(), tx.clone());
            }
            let _ = state.connections_notify.send(());

            // Invia conferma di autenticazione
            let auth_ok = ServerMessage::AuthResult {
                success: true,
                reason: None,
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &auth_ok).await?;

            // Query di prova / messaggio di benvenuto con i dati dell'utente dal DB
            let test_query_msg = ServerMessage::DirectMessage {
                message: format!(
                    "Benvenuto {}, ID utente: {}.",
                    user.username,
                    user.id.as_ref().unwrap_or(&0)
                ),
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &test_query_msg).await?;
        }
        Ok(_) => {
            
            let auth_err = ServerMessage::AuthResult {
                success: false,
                reason: Some("Credenziali non valide".to_string()),
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &auth_err).await?;
        }
        Err(e) => {
            let message = format!("Errore DB durante login di {}: {}", username, e);
            eprintln!("{}", message);
            let _ = state.error_tx.send(crate::state::IncomingError {
                message,
                timestamp: chrono::Utc::now(),
            });
            let err_resp = ServerMessage::Error {
                message: format!(
                    "Errore interno del server durante il recupero utente: {}",
                    e
                ),
                timestamp: chrono::Utc::now(),
                context: ErrorContext::General,
            };
            send_message(writer, &err_resp).await?;
        }
    }
    Ok(())
}
