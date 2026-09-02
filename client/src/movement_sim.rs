use common::models::Position;
// Vera e propria simulazione del movimento che invia i dati di movimento al server (una Position ogni 30 secondi)
use common::{models::PositionWithoutTimestamp, protocol::ClientMessage};
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};

use chrono::Utc;

pub async fn movement_sim(
    positions: Vec<PositionWithoutTimestamp>,
    tx: Sender<ClientMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simulazione del movimento
    let mut timestamp = Utc::now();

    for position in positions {
        let position_with_time = Position {
            lat: position.lat,
            lon: position.lon,
            timestamp,
        };

        // println!("Timestamp: {}", timestamp);

        timestamp = timestamp + chrono::Duration::seconds(30);

        let position_message = ClientMessage::PositionUpdate {
            position: position_with_time,
        };
        if tx.send(position_message).await.is_err() {
            eprintln!("[MOVEMENT_SIM] Impossibile inviare la posizione: canale chiuso");
            break;
        }

        sleep(Duration::from_secs(30)).await;
    }
    Ok(())
}
