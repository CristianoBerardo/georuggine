// Vera e propria simulazione del movimento che invia i dati di movimento al server (una Position ogni 30 secondi)
use common::{models::Position, protocol::ClientMessage};
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};

pub async fn movement_sim(
    positions: Vec<Position>,
    tx: Sender<ClientMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simulazione del movimento
    for position in positions {
        // println!(
        //     "Simulazione movimento: lat: {}, lon: {}, timestamp: {}",
        //     position.lat, position.lon, position.timestamp
        // );

        let position_message = ClientMessage::PositionUpdate { position };
        if tx.send(position_message).await.is_err() {
            eprintln!("[MOVEMENT_SIM] Impossibile inviare la posizione: canale chiuso");
            break;
        }
        sleep(Duration::from_secs(30)).await;
    }
    Ok(())
}
