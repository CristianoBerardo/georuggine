mod support;

use chrono::Utc;
use common::protocol::{ErrorContext, ServerMessage};
use support::{recv, send_raw, send_server_msg};

use crate::support::mock_connection;

#[tokio::test]
async fn errore_server_viene_ricevuto_con_contesto() {
    let ((_, mut server_tx), (mut client_rx, _)) = mock_connection().await;

    send_server_msg(
        &mut server_tx,
        &ServerMessage::Error {
            message: "Non sei autenticato.".to_string(),
            timestamp: Utc::now(),
            context: ErrorContext::Chat,
        },
    )
    .await;

    match recv(&mut client_rx).await {
        ServerMessage::Error {
            message, context, ..
        } => {
            assert_eq!(message, "Non sei autenticato.");
            assert!(matches!(context, ErrorContext::Chat));
        }
        other => panic!("atteso Error, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn listener_termina_quando_il_server_chiude_la_connessione() {
    let ((server_rx, _), (_, _)) = mock_connection().await;

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Se non gestisse EOF non terminerebbe mai
    client::listener::listen(server_rx, tx).await;

    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn messaggio_malformato_dal_server_viene_scartato() {
    let ((_, mut server_tx), (client_rx, _)) = mock_connection().await;

    send_raw(&mut server_tx, "questo non è json valido").await;
    send_server_msg(
        &mut server_tx,
        &ServerMessage::BroadcastMessage {
            message: "valido dopo errore".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    // chiudo server_tx così listen() esce dal loop una volta finiti i dati (EOF)
    drop(server_tx);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Avvio listerner
    client::listener::listen(client_rx, tx).await;

    // Solo messaggi validi, quelli malformati devono essere scartati
    match rx.try_recv().unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "valido dopo errore");
        }
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }

    // Altri canali nel canale?
    assert!(rx.try_recv().is_err());
}
