// Test end-to-end: verificano che le posizioni inviate dal client vengano
// salvate nel database del server e che lo stato utente venga aggiornato.

mod support;

use common::protocol::ServerMessage;
use server::state::UserStatus;
use std::time::Duration;
use support::{HeadlessClient, spawn_test_server};

#[tokio::test]
async fn e2e_posizione_inviata_viene_salvata_nel_database() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Invio posizione
    mario.send_position(45.07, 7.69).await;

    // Polling per attendere che la posizione sia salvata nel DB
    let user = server::db::get_user_by_username(&state.db, "mario")
        .await
        .unwrap()
        .expect("l'utente deve esistere nel DB");

    let mut track_point = None;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        track_point = server::db::get_last_track_point_by_user_id(&state.db, user.id.unwrap())
            .await
            .unwrap();
        if track_point.is_some() {
            break;
        }
    }

    let tp = track_point.expect("il track point deve essere salvato nel DB");
    assert!(tp.lat == 45.07);
    assert!(tp.lon == 7.69);
}

#[tokio::test]
async fn e2e_posizione_cambia_stato_utente_in_movimento() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Registrazione
    let mut mario = HeadlessClient::connect(addr).await;
    match mario.register("mario", "supersegreta").await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Invio posizione
    mario.send_position(45.07, 7.69).await;

    // Polling per attendere che lo stato utente venga aggiornato
    let mut trovato = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let map = state.user_status.read().await;
        if let Some(info) = map.get("mario") {
            if info.status == UserStatus::InMovimento {
                trovato = true;
                break;
            }
        }
    }

    assert!(
        trovato,
        "lo stato dell'utente deve diventare InMovimento dopo un PositionUpdate"
    );
}
