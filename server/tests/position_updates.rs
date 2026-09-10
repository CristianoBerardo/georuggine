mod support;

use common::models::Position;
use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, send, spawn_test_server};

#[tokio::test]
async fn una_posizione_inviata_viene_salvata_nel_database() {
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

    let posizione = Position {
        lat: 45.07,
        lon: 7.69,
        timestamp: chrono::Utc::now(),
    };
    send(
        &mut writer,
        &ClientMessage::PositionUpdate {
            position: posizione.clone(),
        },
    )
    .await;

    // Dato che PositionUpdate non produce nessuna risposta sul socket, si verifica
    // che il punto sia stato salvato direttamente dal db
    let user = server::db::get_user_by_username(&state.db, "mario")
        .await
        .unwrap()
        .expect("mario deve esistere");

    let mut tentativi_rimasti = 20;
    let ultimo_punto = loop {
        if let Some(tp) = server::db::get_last_track_point_by_user_id(&state.db, user.id.unwrap())
            .await
            .unwrap()
        {
            break tp;
        }
        tentativi_rimasti -= 1;
        assert!(
            tentativi_rimasti > 0,
            "il punto di tracciamento doveva comparire nel database"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    };

    assert_eq!(ultimo_punto.lat, posizione.lat);
    assert_eq!(ultimo_punto.lon, posizione.lon);
}

#[tokio::test]
async fn position_update_senza_autenticazione_produce_un_errore() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    let (mut reader, mut writer) = connect(addr).await;
    // Nessun Login/Register prima di questo: la connessione non è autenticata.
    send(
        &mut writer,
        &ClientMessage::PositionUpdate {
            position: Position {
                lat: 45.0,
                lon: 9.0,
                timestamp: chrono::Utc::now(),
            },
        },
    )
    .await;

    match recv(&mut reader).await {
        ServerMessage::Error { message, .. } => {
            assert_eq!(
                message,
                "Devi essere autenticato per inviare aggiornamenti di posizione."
            );
        }
        other => panic!("atteso Error, arrivato {other:?}"),
    }
}
