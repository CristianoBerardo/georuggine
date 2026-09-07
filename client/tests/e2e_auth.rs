// Test end-to-end: verificano il flusso completo di autenticazione
// (Register/Login) dal client headless al server reale con SQLite in-memory.

mod support;

use common::protocol::ServerMessage;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_registrazione_e_login_con_successo() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    // 1. Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
    mario.disconnect().await;

    // 2. Login con le stesse credenziali su una nuova connessione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.login("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // 3. Dopo un login con successo, il server invia un DirectMessage di benvenuto
    match mario.recv().await.unwrap() {
        ServerMessage::DirectMessage { message, .. } => {
            assert!(message.contains("Benvenuto"));
        }
        other => panic!("atteso DirectMessage di benvenuto, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn e2e_login_con_password_sbagliata() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut anna = HeadlessClient::connect(addr).await;
    match anna.register("anna", "password").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
    anna.disconnect().await;

    // Login con password errata
    let mut anna = HeadlessClient::connect(addr).await;
    match anna.login("anna", "password_sbagliata").await {
        ServerMessage::AuthResult { success, .. } => assert!(!success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn e2e_registrazione_username_duplicato() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    // Prima registrazione: deve riuscire
    let mut mario1 = HeadlessClient::connect(addr).await;
    match mario1.register("mario", "prima_password").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Seconda registrazione con lo stesso username: deve fallire
    let mut mario2 = HeadlessClient::connect(addr).await;
    match mario2.register("mario", "altra_password").await {
        ServerMessage::AuthResult {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Username già in uso."));
        }
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}
