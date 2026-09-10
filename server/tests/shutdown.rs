mod support;

use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, send, spawn_test_server};

#[tokio::test]
async fn spegnimento_del_server_avvisa_i_client_connessi() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    let (mut reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: "mario".to_string(),
            password: "password_mario".to_string(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Simuliamo l'operatore che ferma il server
    state
        .shutdown_tx
        .send(())
        .expect("l'invio del segnale di arresto deve riuscire");

    match recv(&mut reader).await {
        ServerMessage::BroadcastMessage { message, .. } => {
            assert_eq!(message, "Il server si sta arrestando, verrai disconnesso.");
        }
        other => panic!("atteso BroadcastMessage, arrivato {other:?}"),
    }
}
