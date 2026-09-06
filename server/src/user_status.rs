use crate::state::{AppState, Info, IncomingError, UserStatus, Username};

fn notify_error(state: &AppState, message: String) {
    let _ = state.error_tx.send(IncomingError {
        message,
        timestamp: chrono::Utc::now(),
    });
}

use crate::db::get_all_users;

pub async fn init_status_map(
    state: &mut AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let users = get_all_users(&state.db).await?;
    for user in users {
        state.user_status.write().await.insert(
            user,
            Info {
                status: UserStatus::Sconnesso,
                s: 0,
            },
        );
    }
    Ok(())
}

pub async fn update_user_status(
    state: &AppState,
    username: &Username,
    new_status: UserStatus,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    if let Some(info) = user_status.get_mut(username) {
        if info.status != new_status {
            info.status = new_status;
            // Notifica la UI che i dati sono cambiati
            let _ = state.connections_notify.send(());
        }
    } else {
        notify_error(state, format!("Utente {} non trovato nella mappa degli stati.", username));
    }
    Ok(())
}

pub async fn update_user_seconds(
    state: &AppState,
    username: &Username,
    seconds: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    if let Some(info) = user_status.get_mut(username) {
        info.s = seconds;
    } else {
        notify_error(state, format!("Utente {} non trovato nella mappa degli stati.", username));
    }
    Ok(())
}

pub async fn get_user_info(
    state: &AppState,
    username: &Username,
) -> Result<Option<Info>, Box<dyn std::error::Error + Send + Sync>> {
    let user_status = state.user_status.read().await;
    Ok(user_status.get(username).cloned())
}

pub async fn remove_user_from_status_map(
    state: &AppState,
    username: &Username,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    user_status.remove(username);
    Ok(())
}

pub async fn add_user_to_status_map(
    state: &AppState,
    username: Username,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    user_status.insert(
        username,
        Info {
            status: UserStatus::Sconnesso,
            s: 0,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{RwLock, mpsc};

    // AppState "vero" (stesso costruttore usato in main.rs) ma con un DB
    // SQLite in memoria mai interrogato: queste funzioni toccano solo
    // `user_status`/`connections_notify`/`error_tx`, non il database.
    // Restituisce anche il ricevitore degli errori, per i test che devono
    // verificare che ne sia stato inviato uno.
    async fn test_state() -> (AppState, mpsc::UnboundedReceiver<IncomingError>) {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        let (connections_notify, _) = tokio::sync::watch::channel(());
        let (chat_tx, _) = mpsc::unbounded_channel();
        let (error_tx, error_rx) = mpsc::unbounded_channel();

        let state = AppState {
            db,
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_status: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
            connections_notify,
            chat_tx,
            error_tx,
        };
        (state, error_rx)
    }

    #[tokio::test]
    async fn utente_aggiunto_si_ritrova_sconnesso_con_zero_secondi() {
        let (state, _error_rx) = test_state().await;

        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();

        let info = get_user_info(&state, &"mario".to_string())
            .await
            .unwrap()
            .expect("l'utente dovrebbe esistere");
        assert_eq!(info.status, UserStatus::Sconnesso);
        assert_eq!(info.s, 0);
    }

    #[tokio::test]
    async fn utente_mai_aggiunto_restituisce_none() {
        let (state, _error_rx) = test_state().await;
        let info = get_user_info(&state, &"fantasma".to_string()).await.unwrap();
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn aggiornare_lo_stato_di_un_utente_esistente_lo_cambia() {
        let (state, _error_rx) = test_state().await;
        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();

        update_user_status(&state, &"mario".to_string(), UserStatus::InMovimento)
            .await
            .unwrap();

        let info = get_user_info(&state, &"mario".to_string())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(info.status, UserStatus::InMovimento);
    }

    #[tokio::test]
    async fn cambiare_stato_notifica_la_ui() {
        let (state, _error_rx) = test_state().await;
        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();
        let notify_rx = state.connections_notify.subscribe();

        update_user_status(&state, &"mario".to_string(), UserStatus::InMovimento)
            .await
            .unwrap();

        assert!(notify_rx.has_changed().unwrap());
    }

    #[tokio::test]
    async fn impostare_lo_stesso_stato_non_notifica_la_ui() {
        let (state, _error_rx) = test_state().await;
        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();
        // Lo stato iniziale è già Sconnesso
        let notify_rx = state.connections_notify.subscribe();

        update_user_status(&state, &"mario".to_string(), UserStatus::Sconnesso)
            .await
            .unwrap();

        assert!(!notify_rx.has_changed().unwrap());
    }

    #[tokio::test]
    async fn aggiornare_lo_stato_di_un_utente_sconosciuto_segnala_un_errore() {
        let (state, mut error_rx) = test_state().await;

        update_user_status(&state, &"fantasma".to_string(), UserStatus::InMovimento)
            .await
            .unwrap();

        let error = error_rx.try_recv().expect("doveva arrivare un errore");
        assert!(error.message.contains("fantasma"));
    }

    #[tokio::test]
    async fn aggiornare_i_secondi_di_un_utente_esistente_li_cambia() {
        let (state, _error_rx) = test_state().await;
        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();

        update_user_seconds(&state, &"mario".to_string(), 42)
            .await
            .unwrap();

        let info = get_user_info(&state, &"mario".to_string())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(info.s, 42);
    }

    #[tokio::test]
    async fn aggiornare_i_secondi_di_un_utente_sconosciuto_segnala_un_errore() {
        let (state, mut error_rx) = test_state().await;

        update_user_seconds(&state, &"fantasma".to_string(), 42)
            .await
            .unwrap();

        let error = error_rx.try_recv().expect("doveva arrivare un errore");
        assert!(error.message.contains("fantasma"));
    }

    #[tokio::test]
    async fn rimuovere_un_utente_esistente_lo_toglie_dalla_mappa() {
        let (state, _error_rx) = test_state().await;
        add_user_to_status_map(&state, "mario".to_string())
            .await
            .unwrap();

        remove_user_from_status_map(&state, &"mario".to_string())
            .await
            .unwrap();

        let info = get_user_info(&state, &"mario".to_string()).await.unwrap();
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn rimuovere_un_utente_mai_aggiunto_non_fallisce() {
        let (state, _error_rx) = test_state().await;

        let result = remove_user_from_status_map(&state, &"fantasma".to_string()).await;

        assert!(result.is_ok());
    }
}
