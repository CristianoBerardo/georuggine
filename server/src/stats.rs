use crate::handlers::handle_connection::WATCHDOG_TIMEOUT_SECS;
use common::models::{MovementStats, TrackPoint};

// Intervallo massimo (in secondi) tra due punti consecutivi perché siano
// considerati parte dello stesso "giro": oltre questa soglia, il buco indica
// che il tracker era spento tra una sessione e l'altra, non un periodo reale
// di marcia o sosta continua, quindi la coppia va ignorata.
const MAX_GAP_SECS: i64 = WATCHDOG_TIMEOUT_SECS as i64;

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

        if p1.lon == p2.lon && p1.lat == p2.lat {
            // Stesso punto: contribuisce solo al tempo di sosta
            paused_duration_secs += elapsed_secs;
            continue;
        } else {
            distance_km += compute_distance_km(p1, p2);
            moving_duration_secs += elapsed_secs;
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn point(lat: f64, lon: f64, secs_offset: i64) -> TrackPoint {
        TrackPoint {
            id: None,
            user_id: 1,
            lat,
            lon,
            timestamp: Utc.timestamp_opt(0, 0).unwrap() + chrono::Duration::seconds(secs_offset),
        }
    }

    #[test]
    fn nessun_punto_restituisce_statistiche_vuote() {
        let stats = compute_stats(&[]);
        assert_eq!(stats.distance_km, 0.0);
        assert_eq!(stats.moving_duration_secs, 0);
        assert_eq!(stats.paused_duration_secs, 0);
    }

    #[test]
    fn punti_uguali_contano_come_sosta() {
        let points = vec![point(45.0, 9.0, 0), point(45.0, 9.0, 30)];
        let stats = compute_stats(&points);
        assert_eq!(stats.paused_duration_secs, 30);
        assert_eq!(stats.moving_duration_secs, 0);
        assert_eq!(stats.distance_km, 0.0);
    }

    #[test]
    fn gap_troppo_grande_viene_ignorato() {
        // MAX_GAP_SECS (= WATCHDOG_TIMEOUT_SECS = 35) è il limite: oltre, la
        // coppia non deve contribuire a nulla
        let points = vec![point(45.0, 9.0, 0), point(45.1, 9.1, 36)];
        let stats = compute_stats(&points);
        assert_eq!(stats.moving_duration_secs, 0);
        assert_eq!(stats.paused_duration_secs, 0);
        assert_eq!(stats.distance_km, 0.0);
    }

    #[test]
    fn movimento_reale_calcola_distanza_e_velocita() {
        // Stessa longitudine, 0.001° di differenza in latitudine: con dlon=0
        // la formula di Haversine si riduce esattamente a
        // EARTH_RADIUS_KM * delta_lat_in_radianti, indipendentemente dalla
        // latitudine di partenza, quindi il valore atteso è calcolabile senza
        // bisogno di una tabella di riferimento esterna.
        let points = vec![point(45.0, 9.0, 0), point(45.001, 9.0, 30)];
        let stats = compute_stats(&points);

        let expected_distance_km = 6371.0 * 0.001_f64.to_radians();
        assert!(
            (stats.distance_km - expected_distance_km).abs() < 1e-6,
            "distanza attesa {expected_distance_km}, ottenuta {}",
            stats.distance_km
        );
        assert_eq!(stats.moving_duration_secs, 30);
        assert_eq!(stats.paused_duration_secs, 0);

        let expected_speed = (expected_distance_km / (30.0 / 3600.0)).round();
        assert_eq!(stats.avg_speed_kmh, expected_speed);
    }
}
