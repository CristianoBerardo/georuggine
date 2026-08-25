use common::models::{MovementStats, TrackPoint, VehicleState};

// Intervallo massimo (in secondi) tra due punti consecutivi perché siano
// considerati parte dello stesso "giro": oltre questa soglia, il buco indica
// che il tracker era spento tra una sessione e l'altra, non un periodo reale
// di marcia o sosta continua, quindi la coppia va ignorata.
const MAX_GAP_SECS: i64 = 30 * 60; // 30 minuti

// Raggio della Terra in km, per la formula di Haversine per il calcolo della
// distanza tra due punti geografici
const EARTH_RADIUS_KM: f64 = 6371.0;

// Funzioni base per la gestione delle statistiche
pub fn compute_stats(points: &[TrackPoint]) -> MovementStats {
    let mut distance_km = 0.0;
    let mut moving_duration_secs: u64 = 0;
    let mut paused_duration_secs: u64 = 0;

    // Scorro i punti a coppie consecutive
    for pair in points.windows(2) {
        let (p1, p2) = (&pair[0], &pair[1]);
        let elapsed_secs = (p2.timestamp - p1.timestamp).num_seconds();

        // Buco troppo grande: punti di sessioni diverse, non contribuiscono
        if elapsed_secs <= 0 || elapsed_secs > MAX_GAP_SECS {
            continue;
        }

        let elapsed_secs = elapsed_secs as u64;
        match p1.state {
            VehicleState::InMovimento => {
                distance_km += compute_distance_km(p1, p2);
                moving_duration_secs += elapsed_secs;
            }
            VehicleState::Fermo => paused_duration_secs += elapsed_secs,
            VehicleState::Sconnesso => {} // Ignora i periodi di disconnessione
        }
    }

    let avg_speed_kmh = if moving_duration_secs > 0 {
        (distance_km / (moving_duration_secs as f64 / 3600.0)).round()
    } else {
        0.0
    };

    MovementStats {
        distance_km,
        avg_speed_kmh,
        moving_duration_secs,
        paused_duration_secs,
    }
}

// Calcolo della distanza con formula di Haversine
fn compute_distance_km(p1: &TrackPoint, p2: &TrackPoint) -> f64 {
    let lat1 = p1.lat.to_radians();
    let lon1 = p1.lon.to_radians();
    let lat2 = p2.lat.to_radians();
    let lon2 = p2.lon.to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}
