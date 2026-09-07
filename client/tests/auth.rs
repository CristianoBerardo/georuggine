mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{recv, recv_client_msg, send_server_msg};

use crate::support::mock_connection;

#[tokio::test]
async fn register_viene_serializzato_e_inviato_correttamente() {
    let ((mut server_rx, mut server_tx), (mut client_rx, mut client_tx)) = mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
        &ClientMessage::Register {
            username: "anna".to_string(),
            password: "password_forte".to_string(),
        },
    )
    .await
    .unwrap();

    // Verifica che il server abbia ricevuto il messaggio

    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::Register { username, password } => {
            assert_eq!(username, "anna");
            assert_eq!(password, "password_forte");
        }
        _ => panic!("Atteso Register"),
    }

    send_server_msg(
        &mut server_tx,
        &ServerMessage::AuthResult {
            success: true,
            reason: None,
            timestamp: Utc::now(),
        },
    )
    .await;

    match recv(&mut client_rx).await {
        ServerMessage::AuthResult {
            success, reason, ..
        } => {
            assert!(success);
            assert!(reason.is_none());
        }
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn auth_result_negativo_viene_ricevuto_con_ragione() {
    let ((mut server_rx, mut server_tx), (mut client_rx, mut client_tx)) = mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
        &ClientMessage::Register {
            username: "duplicato".to_string(),
            password: "qualcosa".to_string(),
        },
    )
    .await
    .unwrap();

    // Verifica che il server abbia ricevuto il messaggio
    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::Register { username, password } => {
            assert_eq!(username, "duplicato");
            assert_eq!(password, "qualcosa");
        }
        _ => panic!("Atteso Register"),
    }

    // Invia la risposta del server
    send_server_msg(
        &mut server_tx,
        &ServerMessage::AuthResult {
            success: false,
            reason: Some("Username già in uso.".to_string()),
            timestamp: Utc::now(),
        },
    )
    .await;

    match recv(&mut client_rx).await {
        ServerMessage::AuthResult {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Username già in uso."));
        }
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}
