use chrono::Utc;
use common::models::Position;
use common::{models::PositionWithoutTimestamp, protocol::ClientMessage};
use tokio::sync::{mpsc::Sender, watch};
use tokio::time::{Duration, sleep};

#[derive(Default, Clone)]
pub struct MovementStatus {
    pub first_sent_position: Option<Position>,
    pub last_sent_position: Option<Position>,
    pub finished: bool,
}

pub async fn movement_sim(
    positions: Vec<PositionWithoutTimestamp>,
    tx: Sender<ClientMessage>,
    status_tx: watch::Sender<MovementStatus>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut timestamp = Utc::now();
    let mut first_sent_position: Option<Position> = None;
    let mut last_sent_position: Option<Position> = None;

    for position in positions {
        let position_with_time = Position {
            lat: position.lat,
            lon: position.lon,
            timestamp,
        };
        timestamp = timestamp + chrono::Duration::seconds(30);

        let position_message = ClientMessage::PositionUpdate {
            position: position_with_time.clone(),
        };
        if tx.send(position_message).await.is_err() {
            eprintln!("[MOVEMENT_SIM] Impossibile inviare la posizione: canale chiuso");
            break;
        }

        if first_sent_position.is_none() {
            first_sent_position = Some(position_with_time.clone());
        }
        last_sent_position = Some(position_with_time);

        let _ = status_tx.send(MovementStatus {
            first_sent_position: first_sent_position.clone(),
            last_sent_position: last_sent_position.clone(),
            finished: false,
        });

        sleep(Duration::from_secs(30)).await;
    }

    let _ = status_tx.send(MovementStatus {
        first_sent_position,
        last_sent_position,
        finished: true,
    });

    Ok(())
}
