// Test end-to-end: verifica che lo spegnimento del server invii un
// messaggio broadcast di avviso a tutti i client connessi.

mod support;

use common::protocol::ServerMessage;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_shutdown_server_avvisa_i_client() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Client connesso e autenticato
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // L'operatore avvia lo shutdown del server
    let _ = state.shutdown_tx.send(());

    // Il client deve ricevere il messaggio di arresto
    match mario.recv().await.unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert!(message.contains("si sta arrestando"));
        }
        other => panic!("atteso BroadcastMessage di shutdown, arrivato {other:?}"),
    }
}
