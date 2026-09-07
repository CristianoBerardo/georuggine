// Test end-to-end: verifica che quando un client si disconnette, il server
// lo rimuove dalla mappa delle connessioni attive.

mod support;

use common::protocol::ServerMessage;
use std::time::Duration;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_client_disconnesso_sparisce_dalle_connessioni() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Verifica che mario sia nelle connessioni attive
    assert!(
        state.connections.read().await.contains_key("mario"),
        "mario deve essere nella mappa delle connessioni dopo la registrazione"
    );

    // Disconnessione
    mario.disconnect().await;

    // Polling: il server deve rilevare la disconnessione e rimuovere mario
    let mut rimosso = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if !state.connections.read().await.contains_key("mario") {
            rimosso = true;
            break;
        }
    }

    assert!(
        rimosso,
        "mario deve sparire dalla mappa delle connessioni dopo la disconnessione"
    );
}
