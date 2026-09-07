mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{recv_client_msg, send_server_msg};

#[tokio::test]
async fn chat_message_viene_inviato_con_timestamp() {
    let now = Utc::now();

    let ((mut server_rx, _), (_, mut client_tx)) = support::mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
        &ClientMessage::ChatMessage {
            message: "ciao a tutti!".to_string(),
            timestamp: now,
        },
    )
    .await
    .unwrap();

    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::ChatMessage { message, timestamp } => {
            assert_eq!(message, "ciao a tutti!");
            assert!(timestamp <= Utc::now());
        }
        other => panic!("atteso ChatMessage, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn broadcast_message_viene_ricevuto_dal_listener() {
    let ((_, mut server_tx), (client_rx, _)) = support::mock_connection().await;

    send_server_msg(
        &mut server_tx,
        &ServerMessage::BroadcastMessage {
            message: "annuncio importante".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    // Chiudiamo così il listener esce dal loop inifito
    drop(server_tx);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(client_rx, tx).await;

    match rx.try_recv().unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "annuncio importante");
        }
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err()); // un solo messaggio
}

#[tokio::test]
async fn direct_message_viene_ricevuto_dal_listener() {
    let ((_, mut server_tx), (client_rx, _)) = support::mock_connection().await;

    send_server_msg(
        &mut server_tx,
        &ServerMessage::DirectMessage {
            message: "messaggio privato".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    // Chiudiamo così il listener esce dal loop inifito
    drop(server_tx);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(client_rx, tx).await;

    match rx.try_recv().unwrap() {
        ServerMessage::DirectMessage { message, .. } => {
            assert_eq!(message, "messaggio privato");
        }
        other => panic!("atteso DirectMessage, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err()); // un solo messaggio
}

#[tokio::test]
async fn piu_messaggi_server_vengono_ricevuti_in_ordine() {
    let ((_, mut server_tx), (client_rx, _)) = support::mock_connection().await;

    send_server_msg(
        &mut server_tx,
        &ServerMessage::BroadcastMessage {
            message: "primo".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;
    send_server_msg(
        &mut server_tx,
        &ServerMessage::DirectMessage {
            message: "secondo".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;
    send_server_msg(
        &mut server_tx,
        &ServerMessage::BroadcastMessage {
            message: "terzo".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    // Chiudiamo così il listener esce dal loop inifito
    drop(server_tx);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(client_rx, tx).await;

    match rx.try_recv().unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "primo"),
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }
    match rx.try_recv().unwrap() {
        ServerMessage::DirectMessage { message, .. } => assert_eq!(message, "secondo"),
        other => panic!("atteso DirectMessage, arrivato {other:?}"),
    }
    match rx.try_recv().unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "terzo"),
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err());
}
