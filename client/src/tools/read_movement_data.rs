// Lettura del file csv contenente i dati che ritorna un vettore di oggetti Position
use common::models::PositionWithoutTimestamp;
use serde::Deserialize;
use std::error::Error;

// Struttura intermedia per deserializzare le righe grezze del CSV con serde.
#[derive(Debug, Deserialize)]
struct MovementRecord {
    pub lat: f64,
    pub lon: f64,
}

// Legge il file CSV specificato, deserializza i record e restituisce un vettore di `Position`.
pub fn read_movement_data(
    file_path: &str,
) -> Result<Vec<PositionWithoutTimestamp>, Box<dyn Error + Send + Sync>> {
    let mut reader_from_file = csv::Reader::from_path(file_path)?;
    let mut positions = Vec::new();

    for result in reader_from_file.deserialize::<MovementRecord>() {
        let record = result?;
        // let timestamp = parse_timestamp(&record.timestamp, base_time)?;

        // println!("Lettura record: timestamp: {}", timestamp);

        positions.push(PositionWithoutTimestamp {
            lat: record.lat,
            lon: record.lon,
        });
    }

    Ok(positions)
}
