mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, recv_client_msg, send_server_msg, spawn_mock_server};

#[tokio::test]
async fn delete_account_viene_inviato_con_password() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, _writer| async move {
        let msg = recv_client_msg(&mut reader).await;
        match msg {
            ClientMessage::DeleteAccount { password } => {
                assert_eq!(password, "la_mia_password");
            }
            other => panic!("atteso DeleteAccount, arrivato {other:?}"),
        }
    })
    .await;

    let (_reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::DeleteAccount {
            password: "la_mia_password".to_string(),
        },
    )
    .await
    .unwrap();

    server_handle.await.unwrap();
}

#[tokio::test]
async fn account_deleted_positivo_viene_ricevuto() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, mut writer| async move {
        let _msg = recv_client_msg(&mut reader).await;
        send_server_msg(
            &mut writer,
            &ServerMessage::AccountDeleted {
                success: true,
                reason: None,
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (mut reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::DeleteAccount {
            password: "password".to_string(),
        },
    )
    .await
    .unwrap();

    match recv(&mut reader).await {
        ServerMessage::AccountDeleted {
            success, reason, ..
        } => {
            assert!(success);
            assert!(reason.is_none());
        }
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }

    server_handle.await.unwrap();
}

#[tokio::test]
async fn account_deleted_negativo_con_ragione_viene_ricevuto() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, mut writer| async move {
        let _msg = recv_client_msg(&mut reader).await;
        send_server_msg(
            &mut writer,
            &ServerMessage::AccountDeleted {
                success: false,
                reason: Some("Password errata.".to_string()),
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (mut reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::DeleteAccount {
            password: "password_sbagliata".to_string(),
        },
    )
    .await
    .unwrap();

    match recv(&mut reader).await {
        ServerMessage::AccountDeleted {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Password errata."));
        }
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }

    server_handle.await.unwrap();
}
