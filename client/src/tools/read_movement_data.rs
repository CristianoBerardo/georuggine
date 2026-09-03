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

// Converte una stringa di timestamp (MM:SS, HH:MM:SS) in un `DateTime<Utc>`.
// fn parse_timestamp(
//     s: &str,
//     base_time: DateTime<Utc>,
// ) -> Result<DateTime<Utc>, Box<dyn Error + Send + Sync>> {
//     let s = s.trim();

//     let parts: Vec<&str> = s.split(':').collect();
//     let offset_secs = match parts.len() {
//         2 => {
//             let mm: i64 = parts[0].trim().parse()?;
//             let ss: i64 = parts[1].trim().parse()?;
//             mm * 60 + ss
//         }
//         3 => {
//             let hh: i64 = parts[0].trim().parse()?;
//             let mm: i64 = parts[1].trim().parse()?;
//             let ss: i64 = parts[2].trim().parse()?;
//             hh * 3600 + mm * 60 + ss
//         }
//         _ => {
//             return Err(format!(
//                 "Formato timestamp non valido (usa formato HH:MM:SS o MM:SS): '{}'",
//                 s
//             )
//             .into());
//         }
//     };

//     Ok(base_time + Duration::seconds(offset_secs))
// }
