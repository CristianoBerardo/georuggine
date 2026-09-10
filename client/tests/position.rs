mod support;

use chrono::Utc;
use common::models::Position;
use common::protocol::ClientMessage;
use support::recv_client_msg;

#[tokio::test]
async fn position_update_viene_inviato_con_coordinate_corrette() {
    let ((mut server_rx, _), (_, mut client_tx)) = support::mock_connection().await;

    client::messaging::send_message(
        &mut client_tx,
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

    let msg = recv_client_msg(&mut server_rx).await;
    match msg {
        ClientMessage::PositionUpdate { position } => {
            assert!(position.lat == 45.07);
            assert!(position.lon == 7.69);
        }
        other => panic!("atteso PositionUpdate, arrivato {other:?}"),
    }
}

#[tokio::test]
async fn position_update_multipli_arrivano_in_sequenza() {
    let ((mut server_rx, _), (_, mut client_tx)) = support::mock_connection().await;

    let now = Utc::now();

    client::messaging::send_message(
        &mut client_tx,
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

    client::messaging::send_message(
        &mut client_tx,
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

    // Prima posizione
    let msg1 = recv_client_msg(&mut server_rx).await;
    match msg1 {
        ClientMessage::PositionUpdate { position } => {
            assert!(position.lat == 45.07);
            assert!(position.lon == 7.69);
        }
        other => panic!("atteso primo PositionUpdate, arrivato {other:?}"),
    }
    // Seconda posizione
    let msg2 = recv_client_msg(&mut server_rx).await;
    match msg2 {
        ClientMessage::PositionUpdate { position } => {
            assert!(position.lat == 41.90);
            assert!(position.lon == 12.49);
        }
        other => panic!("atteso secondo PositionUpdate, arrivato {other:?}"),
    }
}
