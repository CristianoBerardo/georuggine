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
            // Rifiuta il login se l'utente ha già una sessione attiva su un'altra connessione,
            // per evitare che due dispositivi inviino posizioni contemporaneamente sotto lo stesso utente.
            {
                let mut conns = state.connections.write().await;
                // Una richiesta di login proveniente dalla stessa connessione già
                // registrata per questo utente (stesso canale) non è un duplicato:
                // succede ad es. dopo una registrazione seguita da un login esplicito.
                let is_other_connection = conns
                    .get(&username)
                    .is_some_and(|existing_tx| !existing_tx.same_channel(tx));
                if is_other_connection {
                    drop(conns);
                    let auth_err = ServerMessage::AuthResult {
                        success: false,
                        reason: Some("Utente già connesso da un altro dispositivo".to_string()),
                        timestamp: chrono::Utc::now(),
                    };
                    send_message(writer, &auth_err).await?;
                    return Ok(());
                }
                conns.insert(username.clone(), tx.clone());
            }
            *authenticated_user = Some(username.clone());
            let _ = state.connections_notify.send(());

            // Invia conferma di autenticazione
            let auth_ok = ServerMessage::AuthResult {
                success: true,
                reason: None,
                timestamp: chrono::Utc::now(),
            };
            send_message(writer, &auth_ok).await?;
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
