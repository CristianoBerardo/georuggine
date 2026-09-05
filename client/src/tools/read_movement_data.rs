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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // File temporanei isolati dai dati reali del client: un nome distinto
    // per test evita conflitti quando `cargo test` li esegue in parallelo.
    fn temp_csv(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("georuggine_test_{}.csv", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn legge_posizioni_da_csv_valido() {
        let path = temp_csv(
            "legge_posizioni_da_csv_valido",
            "lat,lon\n45.0,9.0\n46.5,10.25\n",
        );

        let positions = read_movement_data(path.to_str().unwrap()).unwrap();

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].lat, 45.0);
        assert_eq!(positions[0].lon, 9.0);
        assert_eq!(positions[1].lat, 46.5);
        assert_eq!(positions[1].lon, 10.25);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn csv_con_sole_intestazioni_da_vettore_vuoto() {
        let path = temp_csv("csv_con_sole_intestazioni_da_vettore_vuoto", "lat,lon\n");

        let positions = read_movement_data(path.to_str().unwrap()).unwrap();

        assert!(positions.is_empty());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn colonne_in_ordine_diverso_vengono_lette_comunque() {
        // Il csv crate mappa le colonne per nome, non per posizione
        let path = temp_csv(
            "colonne_in_ordine_diverso_vengono_lette_comunque",
            "lon,lat\n9.0,45.0\n",
        );

        let positions = read_movement_data(path.to_str().unwrap()).unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].lat, 45.0);
        assert_eq!(positions[0].lon, 9.0);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn colonna_mancante_restituisce_errore() {
        let path = temp_csv("colonna_mancante_restituisce_errore", "lat\n45.0\n");

        let result = read_movement_data(path.to_str().unwrap());

        assert!(result.is_err());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_inesistente_restituisce_errore() {
        let result = read_movement_data("/percorso/che/non/esiste.csv");
        assert!(result.is_err());
    }
}
