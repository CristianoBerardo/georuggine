use chrono::Utc;
use common::models::Position;
use common::{models::PositionWithoutTimestamp, protocol::ClientMessage};
use tokio::sync::{mpsc::Sender, watch};
use tokio::time::{Duration, sleep};

#[derive(Default, Clone, PartialEq)]
pub enum MovementState {
    #[default]
    InMovimento,
    Fermo,
    Problema,
}

#[derive(Default, Clone)]
pub struct MovementStatus {
    pub first_sent_position: Option<Position>,
    pub last_sent_position: Option<Position>,
    pub state: MovementState,
}

const MAX_STATIONARY_SECS: i64 = 180;

pub async fn movement_sim(
    positions: Vec<PositionWithoutTimestamp>,
    tx: Sender<ClientMessage>,
    status_tx: watch::Sender<MovementStatus>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut timestamp = Utc::now();
    let mut first_sent_position: Option<Position> = None;
    let mut last_sent_position: Option<Position> = None;
    let mut stationary_secs: i64 = 0;

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

        let is_same_position = last_sent_position
            .as_ref()
            .map(|p| p.lat == position_with_time.lat && p.lon == position_with_time.lon)
            .unwrap_or(false);

        stationary_secs = if is_same_position {
            stationary_secs + 30
        } else {
            0
        };

        if first_sent_position.is_none() {
            first_sent_position = Some(position_with_time.clone());
        }
        last_sent_position = Some(position_with_time);

        let state = if stationary_secs >= MAX_STATIONARY_SECS {
            MovementState::Fermo
        } else {
            MovementState::InMovimento
        };

        let _ = status_tx.send(MovementStatus {
            first_sent_position: first_sent_position.clone(),
            last_sent_position: last_sent_position.clone(),
            state,
        });

        sleep(Duration::from_secs(30)).await;
    }

    // Il loop è finito (CSV esaurito o canale rotto, non distinguiamo): non stai più inviando.
    let _ = status_tx.send(MovementStatus {
        first_sent_position,
        last_sent_position,
        state: MovementState::Problema,
    });

    Ok(())
}
