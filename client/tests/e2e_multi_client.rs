// Test end-to-end: verificano scenari multi-client — messaggi diretti
// dall'operatore a un singolo utente e broadcast a tutti gli utenti connessi.

mod support;

use chrono::Utc;
use common::protocol::ServerMessage;
use std::time::Duration;
use support::{spawn_test_server, HeadlessClient};

#[tokio::test]
async fn e2e_messaggio_diretto_arriva_solo_al_destinatario() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Due client connessi
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult per mario, arrivato {other:?}"),
    }

    let mut anna = HeadlessClient::connect(addr).await;
    match anna.register("anna", "password").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult per anna, arrivato {other:?}"),
    }

    // L'operatore invia un messaggio diretto solo a mario
    {
        let connections = state.connections.read().await;
        let mario_tx = connections.get("mario").expect("mario deve essere connesso");
        mario_tx
            .send(ServerMessage::DirectMessage {
                message: "messaggio solo per te".to_string(),
                timestamp: Utc::now(),
            })
            .unwrap();
    }

    // Mario deve ricevere il messaggio
    match mario.recv().await.unwrap() {
        ServerMessage::DirectMessage { message, .. } => {
            assert_eq!(message, "messaggio solo per te");
        }
        other => panic!("atteso DirectMessage per mario, arrivato {other:?}"),
    }

    // Anna NON deve ricevere nulla
    let nulla = anna.try_recv_timeout(Duration::from_millis(200)).await;
    assert!(
        nulla.is_none(),
        "anna non deve ricevere il messaggio diretto a mario"
    );
}

#[tokio::test]
async fn e2e_broadcast_arriva_a_tutti() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Due client connessi
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult per mario, arrivato {other:?}"),
    }

    let mut anna = HeadlessClient::connect(addr).await;
    match anna.register("anna", "password").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult per anna, arrivato {other:?}"),
    }

    // L'operatore invia un broadcast a tutti
    {
        let msg = ServerMessage::BroadcastMessage {
            message: "annuncio per tutti".to_string(),
            timestamp: Utc::now(),
        };
        let connections = state.connections.read().await;
        for sender in connections.values() {
            let _ = sender.send(msg.clone());
        }
    }

    // Entrambi devono ricevere il messaggio
    match mario.recv().await.unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "annuncio per tutti");
        }
        other => panic!("atteso BroadcastMessage per mario, arrivato {other:?}"),
    }

    match anna.recv().await.unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "annuncio per tutti");
        }
        other => panic!("atteso BroadcastMessage per anna, arrivato {other:?}"),
    }
}
