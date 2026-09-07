// Test end-to-end: verificano il flusso completo di eliminazione account
// dal client headless al server reale.

mod support;

use common::protocol::ServerMessage;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_eliminazione_account_e_login_successivo_fallisce() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    // 1. Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // 2. Eliminazione account con password corretta
    match mario.delete_account("supersegreta").await {
        ServerMessage::AccountDeleted { success, .. } => assert!(success),
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }
    mario.disconnect().await;

    // 3. Login con le stesse credenziali su una nuova connessione: deve fallire
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.login("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(!success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn e2e_eliminazione_account_con_password_sbagliata_fallisce() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Eliminazione con password errata: deve fallire
    match mario.delete_account("password_sbagliata").await {
        ServerMessage::AccountDeleted {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Password errata."));
        }
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }
}
