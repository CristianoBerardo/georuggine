mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, recv_client_msg, send_server_msg, spawn_mock_server};

#[tokio::test]
async fn login_viene_serializzato_e_inviato_correttamente() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, mut writer| async move {
        let msg = recv_client_msg(&mut reader).await;
        match msg {
            ClientMessage::Login { username, password } => {
                assert_eq!(username, "mario");
                assert_eq!(password, "segreta123");
            }
            other => panic!("atteso Login, arrivato {other:?}"),
        }
        // Risponde con successo
        send_server_msg(
            &mut writer,
            &ServerMessage::AuthResult {
                success: true,
                reason: None,
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (mut reader, mut writer) = connect(addr).await;

    // uso login vero definito in messaging
    client::messaging::send_message(
        &mut writer,
        &ClientMessage::Login {
            username: "mario".to_string(),
            password: "segreta123".to_string(),
        },
    )
    .await
    .unwrap();


    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    server_handle.await.unwrap();
}

#[tokio::test]
async fn register_viene_serializzato_e_inviato_correttamente() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, mut writer| async move {
        let msg = recv_client_msg(&mut reader).await;
        match msg {
            ClientMessage::Register { username, password } => {
                assert_eq!(username, "anna");
                assert_eq!(password, "password_forte");
            }
            other => panic!("atteso Register, arrivato {other:?}"),
        }
        send_server_msg(
            &mut writer,
            &ServerMessage::AuthResult {
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
        &ClientMessage::Register {
            username: "anna".to_string(),
            password: "password_forte".to_string(),
        },
    )
    .await
    .unwrap();

    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    server_handle.await.unwrap();
}

#[tokio::test]
async fn auth_result_negativo_viene_ricevuto_con_ragione() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, mut writer| async move {

        let _msg = recv_client_msg(&mut reader).await;
        // AuthResult negativo
        send_server_msg(
            &mut writer,
            &ServerMessage::AuthResult {
                success: false,
                reason: Some("Username già in uso.".to_string()),
                timestamp: Utc::now(),
            },
        )
        .await;
    })
    .await;

    let (mut reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::Register {
            username: "duplicato".to_string(),
            password: "qualcosa".to_string(),
        },
    )
    .await
    .unwrap();

    match recv(&mut reader).await {
        ServerMessage::AuthResult {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Username già in uso."));
        }
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    server_handle.await.unwrap();
}
