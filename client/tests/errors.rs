mod support;

use chrono::Utc;
use common::protocol::{ErrorContext, ServerMessage};
use support::{connect, send_raw, send_server_msg, spawn_mock_server};

#[tokio::test]
async fn errore_server_viene_ricevuto_con_contesto() {
    let (addr, _server_handle) = spawn_mock_server(|_reader, mut writer| async move {
        send_server_msg(
            &mut writer,
            &ServerMessage::Error {
                message: "Non sei autenticato.".to_string(),
                timestamp: Utc::now(),
                context: ErrorContext::Chat,
            },
        )
        .await;
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(reader, tx).await;

    match rx.try_recv().unwrap() {
        ServerMessage::Error {
            message, context, ..
        } => {
            assert_eq!(message, "Non sei autenticato.");
            assert!(matches!(context, ErrorContext::Chat));
        }
        other => panic!("atteso Error, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn listener_termina_quando_il_server_chiude_la_connessione() {
    let (addr, _server_handle) = spawn_mock_server(|_reader, _writer| async move {
        // Il mock server chiude immediatamente la connessione: drop socket
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Se il listener non gestisse l'EOF, questo await non finirebbe mai:
    // se termina il test ok
    client::listener::listen(reader, tx).await;

    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn messaggio_malformato_dal_server_viene_scartato() {
    let (addr, _server_handle) = spawn_mock_server(|_reader, mut writer| async move {
        // Invia prima un JSON malformato
        send_raw(&mut writer, "questo non è json valido").await;
        // Poi invia un messaggio valido
        send_server_msg(
            &mut writer,
            &ServerMessage::BroadcastMessage {
                message: "valido dopo errore".to_string(),
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (reader, _writer) = connect(addr).await;
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    client::listener::listen(reader, tx).await;

    // Solo il messaggio valido deve essere stato ricevuto, il malformato scartato
    match rx.try_recv().unwrap() {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "valido dopo errore");
        }
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }
    assert!(rx.try_recv().is_err());
}
