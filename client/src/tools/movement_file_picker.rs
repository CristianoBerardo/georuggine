use std::fs::{read_to_string, write};

const MOVEMENT_FILES: [&str; 3] = [
    "client/src/movement_data/torino-asti.csv",
    "client/src/movement_data/terni-basilicata.csv",
    "client/src/movement_data/dati-prof.csv",
];

const INDEX_FILE: &str = "client/.movement_index";

pub fn next_movement_file() -> &'static str {
    let current = read_to_string(INDEX_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let file = MOVEMENT_FILES[current % MOVEMENT_FILES.len()];
    let next = (current + 1) % MOVEMENT_FILES.len();
    write(INDEX_FILE, next.to_string()).unwrap_or_else(|_| ());
    file
}
