use chrono::Utc;
use common::models::Position;
use common::{models::PositionWithoutTimestamp, protocol::ClientMessage};
use tokio::sync::{mpsc::Sender, watch};
use tokio::time::{Duration, sleep};

#[derive(Debug, Default, Clone, PartialEq)]
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

// Dato se la posizione corrente coincide con l'ultima inviata e da quanti
// secondi si è eventualmente fermi, calcola il nuovo tempo di sosta e lo
// stato risultante. Isolata dal loop di `movement_sim` per poterla testare
// senza dover simulare l'attesa di 30 secondi tra un invio e l'altro.
fn stationary_state(is_same_position: bool, previous_stationary_secs: i64) -> (i64, MovementState) {
    let stationary_secs = if is_same_position {
        previous_stationary_secs + 30
    } else {
        0
    };

    let state = if stationary_secs >= MAX_STATIONARY_SECS {
        MovementState::Fermo
    } else {
        MovementState::InMovimento
    };

    (stationary_secs, state)
}

pub async fn movement_sim(
    positions: Vec<PositionWithoutTimestamp>,
    tx: Sender<ClientMessage>,
    status_tx: watch::Sender<MovementStatus>,
    error_tx: tokio::sync::mpsc::UnboundedSender<String>,
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
        timestamp += chrono::Duration::seconds(30);

        let position_message = ClientMessage::PositionUpdate {
            position: position_with_time.clone(),
        };
        if tx.send(position_message).await.is_err() {
            let message = "[MOVEMENT_SIM] Impossibile inviare la posizione: canale chiuso".to_string();
            let _ = error_tx.send(message);
            break;
        }

        let is_same_position = last_sent_position
            .as_ref()
            .map(|p| p.lat == position_with_time.lat && p.lon == position_with_time.lon)
            .unwrap_or(false);

        let (new_stationary_secs, state) = stationary_state(is_same_position, stationary_secs);
        stationary_secs = new_stationary_secs;

        if first_sent_position.is_none() {
            first_sent_position = Some(position_with_time.clone());
        }
        last_sent_position = Some(position_with_time);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn point(lat: f64, lon: f64) -> PositionWithoutTimestamp {
        PositionWithoutTimestamp { lat, lon }
    }

    // --- stationary_state ---

    #[test]
    fn posizione_diversa_azzera_il_conteggio() {
        assert_eq!(stationary_state(false, 150), (0, MovementState::InMovimento));
    }

    #[test]
    fn prima_posizione_senza_precedente_e_in_movimento() {
        assert_eq!(stationary_state(false, 0), (0, MovementState::InMovimento));
    }

    #[test]
    fn stessa_posizione_accumula_30_secondi_alla_volta() {
        assert_eq!(stationary_state(true, 60), (90, MovementState::InMovimento));
    }

    #[test]
    fn appena_sotto_la_soglia_resta_in_movimento() {
        assert_eq!(stationary_state(true, 120), (150, MovementState::InMovimento));
    }

    #[test]
    fn al_raggiungimento_della_soglia_diventa_fermo() {
        assert_eq!(stationary_state(true, 150), (180, MovementState::Fermo));
    }

    #[test]
    fn oltre_la_soglia_resta_fermo() {
        assert_eq!(stationary_state(true, 180), (210, MovementState::Fermo));
    }

    // --- movement_sim ---

    #[tokio::test(start_paused = true)]
    async fn invia_una_position_update_per_ogni_posizione_a_30_secondi_di_distanza() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let (status_tx, _status_rx) = watch::channel(MovementStatus::default());
        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let positions = vec![point(45.0, 9.0), point(46.0, 10.0)];

        movement_sim(positions, tx, status_tx, error_tx)
            .await
            .unwrap();

        let ClientMessage::PositionUpdate { position: first } = rx.try_recv().unwrap() else {
            panic!("atteso un PositionUpdate");
        };
        let ClientMessage::PositionUpdate { position: second } = rx.try_recv().unwrap() else {
            panic!("atteso un PositionUpdate");
        };
        assert!(rx.try_recv().is_err()); // solo due posizioni, nessun'altra

        assert_eq!((first.lat, first.lon), (45.0, 9.0));
        assert_eq!((second.lat, second.lon), (46.0, 10.0));
        assert_eq!(second.timestamp - first.timestamp, chrono::Duration::seconds(30));
    }

    #[tokio::test(start_paused = true)]
    async fn a_percorso_esaurito_lo_stato_finale_e_problema() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let (status_tx, status_rx) = watch::channel(MovementStatus::default());
        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let positions = vec![point(45.0, 9.0), point(45.0, 9.0)]; // ferma per tutto il tragitto

        movement_sim(positions, tx, status_tx, error_tx)
            .await
            .unwrap();
        while rx.try_recv().is_ok() {} // svuota il canale, non serve qui

        let status = status_rx.borrow();
        assert_eq!(status.state, MovementState::Problema);
        assert_eq!(status.first_sent_position.as_ref().unwrap().lat, 45.0);
        assert_eq!(status.last_sent_position.as_ref().unwrap().lat, 45.0);
    }

    #[tokio::test(start_paused = true)]
    async fn canale_di_invio_chiuso_interrompe_la_simulazione_e_segnala_un_errore() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        drop(rx); // nessuno riceverà mai i messaggi
        let (status_tx, _status_rx) = watch::channel(MovementStatus::default());
        let (error_tx, mut error_rx) = tokio::sync::mpsc::unbounded_channel();
        let positions = vec![point(45.0, 9.0), point(46.0, 10.0)];

        movement_sim(positions, tx, status_tx, error_tx)
            .await
            .unwrap();

        let error = error_rx.try_recv().expect("doveva arrivare un errore");
        assert!(error.contains("canale chiuso"));
    }
}
