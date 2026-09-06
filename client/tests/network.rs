// Test di integrazione: verificano la parte di rete del client
// (client::messaging / client::listener) contro un server reale o fittizio.

use client::tools::read_movement_data::read_movement_data;

#[test]
fn placeholder_verifica_laggancio_alla_libreria() {
    // Un path inesistente deve fallire in modo pulito, non panicare
    assert!(read_movement_data("/percorso/che/non/esiste.csv").is_err());
}
