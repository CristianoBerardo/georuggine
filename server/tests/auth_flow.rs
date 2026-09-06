// Test di integrazione: verificano il comportamento end-to-end del server
// reale (rete + database), collegandosi via TCP come farebbe un client vero.

mod support;

use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, send, spawn_test_server};

#[tokio::test]
async fn registrazione_eliminazione_account_e_login_successivo_fallito() {
    let addr = spawn_test_server().await;
    let username = "mario_test".to_string();
    let password = "supersegreta".to_string();

    // 1. Registrazione
    let (mut reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: username.clone(),
            password: password.clone(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // 2. Eliminazione account con password sbagliata: deve fallire
    send(
        &mut writer,
        &ClientMessage::DeleteAccount {
            password: "password_sbagliata".to_string(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AccountDeleted { success, .. } => assert!(!success),
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }

    // 3. Eliminazione account con password corretta: deve riuscire
    send(
        &mut writer,
        &ClientMessage::DeleteAccount {
            password: password.clone(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AccountDeleted { success, .. } => assert!(success),
        other => panic!("atteso AccountDeleted, arrivato {other:?}"),
    }

    // 4. Nuova connessione, login con le stesse credenziali: deve fallire
    let (mut reader2, mut writer2) = connect(addr).await;
    send(&mut writer2, &ClientMessage::Login { username, password }).await;
    match recv(&mut reader2).await {
        ServerMessage::AuthResult { success, .. } => assert!(!success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn login_con_password_sbagliata_poi_corretta() {
    let addr = spawn_test_server().await;
    let username = "anna_test".to_string();
    let password = "password_giusta".to_string();

    // Registrazione
    let (mut reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: username.clone(),
            password: password.clone(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Login con password sbagliata: deve fallire
    send(
        &mut writer,
        &ClientMessage::Login {
            username: username.clone(),
            password: "password_sbagliata".to_string(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(!success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Login con la password corretta: deve riuscire
    send(&mut writer, &ClientMessage::Login { username, password }).await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn registrazione_con_username_duplicato_viene_rifiutata() {
    let addr = spawn_test_server().await;
    let username = "duplicato_test".to_string();

    // Prima registrazione: deve riuscire
    let (mut reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: username.clone(),
            password: "prima_password".to_string(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Seconda registrazione con lo stesso username (nuova connessione,
    // password diversa): deve fallire
    let (mut reader2, mut writer2) = connect(addr).await;
    send(
        &mut writer2,
        &ClientMessage::Register {
            username,
            password: "altra_password".to_string(),
        },
    )
    .await;
    match recv(&mut reader2).await {
        ServerMessage::AuthResult {
            success, reason, ..
        } => {
            assert!(!success);
            assert_eq!(reason.as_deref(), Some("Username già in uso."));
        }
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }
}
