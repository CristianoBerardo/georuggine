mod support;

use chrono::Utc;
use common::models::Position;
use common::protocol::ClientMessage;
use support::{connect, recv_client_msg, spawn_mock_server};

#[tokio::test]
async fn position_update_viene_inviato_con_coordinate_corrette() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, _writer| async move {
        let msg = recv_client_msg(&mut reader).await;
        match msg {
            ClientMessage::PositionUpdate { position } => {
                assert!((position.lat - 45.07).abs() < f64::EPSILON);
                assert!((position.lon - 7.69).abs() < f64::EPSILON);
            }
            other => panic!("atteso PositionUpdate, arrivato {other:?}"),
        }
    })
    .await;

    let (_reader, mut writer) = connect(addr).await;

    client::messaging::send_message(
        &mut writer,
        &ClientMessage::PositionUpdate {
            position: Position {
                lat: 45.07,
                lon: 7.69,
                timestamp: Utc::now(),
            },
        },
    )
    .await
    .unwrap();

    server_handle.await.unwrap();
}

#[tokio::test]
async fn position_update_multipli_arrivano_in_sequenza() {
    let (addr, server_handle) = spawn_mock_server(|mut reader, _writer| async move {
        // Prima posizione
        let msg1 = recv_client_msg(&mut reader).await;
        match msg1 {
            ClientMessage::PositionUpdate { position } => {
                assert!((position.lat - 45.07).abs() < f64::EPSILON);
                assert!((position.lon - 7.69).abs() < f64::EPSILON);
            }
            other => panic!("atteso primo PositionUpdate, arrivato {other:?}"),
        }
        // Seconda posizione
        let msg2 = recv_client_msg(&mut reader).await;
        match msg2 {
            ClientMessage::PositionUpdate { position } => {
                assert!((position.lat - 41.90).abs() < f64::EPSILON);
                assert!((position.lon - 12.49).abs() < f64::EPSILON);
            }
            other => panic!("atteso secondo PositionUpdate, arrivato {other:?}"),
        }
    })
    .await;

    let (_reader, mut writer) = connect(addr).await;

    let now = Utc::now();

    // Torino
    client::messaging::send_message(
        &mut writer,
        &ClientMessage::PositionUpdate {
            position: Position {
                lat: 45.07,
                lon: 7.69,
                timestamp: now,
            },
        },
    )
    .await
    .unwrap();

    // Roma
    client::messaging::send_message(
        &mut writer,
        &ClientMessage::PositionUpdate {
            position: Position {
                lat: 41.90,
                lon: 12.49,
                timestamp: now + chrono::Duration::seconds(30),
            },
        },
    )
    .await
    .unwrap();

    server_handle.await.unwrap();
}
