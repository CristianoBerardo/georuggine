use std::fs::{read_to_string, write};

const MOVEMENT_FILES: [&str; 3] = [
    "client/src/movement_data/torino-asti.csv",
    "client/src/movement_data/terni-basilicata.csv",
    "client/src/movement_data/dati-prof.csv",
];

const INDEX_FILE: &str = "client/.movement_index";

pub fn next_movement_file() -> &'static str {
    next_from(&MOVEMENT_FILES, INDEX_FILE)
}

// Stessa logica di `next_movement_file`, ma con elenco file e percorso
// dell'indice parametrizzati: permette ai test di usare un file di indice
// temporaneo invece di quello vero usato dal client in esecuzione.
fn next_from(files: &[&'static str], index_file: &str) -> &'static str {
    let current = read_to_string(index_file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let file = files[current % files.len()];
    let next = (current + 1) % files.len();
    write(index_file, next.to_string()).unwrap_or(());
    file
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILES: [&str; 3] = ["a.csv", "b.csv", "c.csv"];

    // Percorso temporaneo isolato dal vero `client/.movement_index`, distinto
    // per test per evitare conflitti quando `cargo test` li esegue in parallelo.
    fn temp_index(name: &str) -> String {
        let path = std::env::temp_dir().join(format!("georuggine_test_index_{}", name));
        let _ = std::fs::remove_file(&path); // niente indice residuo da run precedenti
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn senza_indice_pregresso_parte_dal_primo_file() {
        let index_file = temp_index("senza_indice_pregresso_parte_dal_primo_file");
        assert_eq!(next_from(&FILES, &index_file), "a.csv");
        std::fs::remove_file(&index_file).unwrap();
    }

    #[test]
    fn chiamate_successive_ruotano_tra_i_file_in_ordine() {
        let index_file = temp_index("chiamate_successive_ruotano_tra_i_file_in_ordine");
        assert_eq!(next_from(&FILES, &index_file), "a.csv");
        assert_eq!(next_from(&FILES, &index_file), "b.csv");
        assert_eq!(next_from(&FILES, &index_file), "c.csv");
        std::fs::remove_file(&index_file).unwrap();
    }

    #[test]
    fn dopo_lultimo_file_si_torna_al_primo() {
        let index_file = temp_index("dopo_lultimo_file_si_torna_al_primo");
        for _ in 0..FILES.len() {
            next_from(&FILES, &index_file);
        }
        assert_eq!(next_from(&FILES, &index_file), "a.csv");
        std::fs::remove_file(&index_file).unwrap();
    }

    #[test]
    fn lindice_viene_persistito_tra_chiamate_diverse() {
        let index_file = temp_index("lindice_viene_persistito_tra_chiamate_diverse");
        next_from(&FILES, &index_file); // scrive l'indice 1 su file

        // Una "nuova esecuzione" che rilegge lo stesso file di indice deve
        // ripartire da dove l'altra aveva lasciato, non da capo.
        assert_eq!(next_from(&FILES, &index_file), "b.csv");
        std::fs::remove_file(&index_file).unwrap();
    }

    #[test]
    fn contenuto_non_numerico_nellindice_viene_ignorato() {
        let index_file = temp_index("contenuto_non_numerico_nellindice_viene_ignorato");
        std::fs::write(&index_file, "non-un-numero").unwrap();

        assert_eq!(next_from(&FILES, &index_file), "a.csv");
        std::fs::remove_file(&index_file).unwrap();
    }
}
