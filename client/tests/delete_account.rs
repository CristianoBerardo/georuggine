mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{recv, recv_client_msg, send_server_msg};

#[tokio::test]
async fn account_deleted_viene_inviato_e_msg_positivo_viene_ricevuto() {
    let ((mut server_rx, mut server_tx), (mut client_rx, mut client_tx)) =
        support::mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
        &ClientMessage::DeleteAccount {
            password: "password".to_string(),
        },
    )
    .await
    .unwrap();

    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::DeleteAccount { password } => {
            assert_eq!(password, "password");
        }
        _ => panic!("Atteso DeleteAccount"),
    }

    send_server_msg(
        &mut server_tx,
        &ServerMessage::AccountDeleted {
            success: true,
            reason: None,
            timestamp: Utc::now(),
        },
    )
    .await;

    match recv(&mut client_rx).await {
        ServerMessage::AccountDeleted {
            success, reason, ..
        } => {
            assert!(success);
            assert!(reason.is_none());
        }
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn account_deleted_negativo_con_ragione_viene_ricevuto() {
    let ((mut server_rx, mut server_tx), (mut client_rx, mut client_tx)) =
        support::mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
        &ClientMessage::DeleteAccount {
            password: "password_sbagliata".to_string(),
        },
    )
    .await
    .unwrap();

    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::DeleteAccount { password } => {
            assert_eq!(password, "password_sbagliata");
        }
        _ => panic!("Atteso DeleteAccount"),
    }

    send_server_msg(
        &mut server_tx,
        &ServerMessage::AccountDeleted {
            success: false,
            reason: Some("Password errata.".to_string()),
            timestamp: Utc::now(),
        },
    )
    .await;

    match recv(&mut client_rx).await {
        ServerMessage::AccountDeleted {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Password errata."));
        }
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }
}
