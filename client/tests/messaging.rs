mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv_client_msg, send_server_msg, spawn_mock_server};

#[tokio::test]
async fn chat_message_viene_inviato_con_timestamp() {
    let now = Utc::now();

    let (addr, server_handle) = spawn_mock_server(|mut reader, _writer| async move {
        let msg = recv_client_msg(&mut reader).await;
        match msg {
            ClientMessage::ChatMessage { message, timestamp } => {
                assert_eq!(message, "ciao a tutti!");
                assert!(timestamp <= Utc::now());
            }
            other => panic!("atteso ChatMessage, arrivato {other:?}"),
        }
    })
    .await;

    let (_reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::ChatMessage {
            message: "ciao a tutti!".to_string(),
            timestamp: now,
        },
    )
    .await
    .unwrap();

    server_handle.await.unwrap();
}

#[tokio::test]
async fn broadcast_message_viene_ricevuto_dal_listener() {
    let (addr, _server_handle) = spawn_mock_server(|_reader, mut writer| async move {
        send_server_msg(
            &mut writer,
            &ServerMessage::BroadcastMessage {
                message: "annuncio importante".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
        // drop del writer causa EOF -> il listener termina
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(reader, tx).await;

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
    let (addr, _server_handle) = spawn_mock_server(|_reader, mut writer| async move {
        send_server_msg(
            &mut writer,
            &ServerMessage::DirectMessage {
                message: "messaggio privato".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(reader, tx).await;

    match rx.try_recv().unwrap() {
        ServerMessage::DirectMessage { message, .. } => {
            assert_eq!(message, "messaggio privato");
        }
        other => panic!("atteso DirectMessage, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn piu_messaggi_server_vengono_ricevuti_in_ordine() {
    let (addr, _server_handle) = spawn_mock_server(|_reader, mut writer| async move {
        send_server_msg(
            &mut writer,
            &ServerMessage::BroadcastMessage {
                message: "primo".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
        send_server_msg(
            &mut writer,
            &ServerMessage::DirectMessage {
                message: "secondo".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
        send_server_msg(
            &mut writer,
            &ServerMessage::BroadcastMessage {
                message: "terzo".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(reader, tx).await;

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
