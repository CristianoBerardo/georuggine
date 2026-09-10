// Test end-to-end: verificano che i messaggi chat del client arrivino
// all'operatore del server tramite il canale chat_rx.

mod support;

use common::protocol::ServerMessage;
use std::time::Duration;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_chat_del_client_arriva_alloperatore() {
    let (addr, _state, mut chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Invio chat
    mario.send_chat("ciao operatore!").await;

    // L'operatore deve ricevere il messaggio
    let chat = tokio::time::timeout(Duration::from_secs(2), chat_rx.recv())
        .await
        .expect("timeout: il messaggio chat non è arrivato all'operatore")
        .expect("canale chat chiuso");

    assert_eq!(chat.from_username, "mario");
    assert_eq!(chat.message, "ciao operatore!");
}

#[tokio::test]
async fn e2e_chat_da_non_autenticato_viene_ignorata() {
    let (addr, _state, mut chat_rx) = spawn_test_server().await;

    // Connessione senza autenticazione
    let mut client = HeadlessClient::connect(addr).await;
    client.send_chat("messaggio fantasma").await;

    // L'operatore NON deve ricevere nulla (timeout)
    let risultato = tokio::time::timeout(Duration::from_millis(300), chat_rx.recv()).await;
    assert!(
        risultato.is_err(),
        "il messaggio di un client non autenticato non deve arrivare all'operatore"
    );
}
