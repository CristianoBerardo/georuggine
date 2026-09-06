// Test di integrazione: verificano il comportamento end-to-end del server
// reale (rete + database), collegandosi via TCP come farebbe un client vero.
//

// Scenari di test:
// account con password sbagliata/corretta, login fallito dopo la
// cancellazione. Per ora solo uno smoke test che dimostra che il crate
// `server` è raggiungibile come libreria da qui.

use server::auth::{hash_password, verify_password};

#[test]
fn placeholder_verifica_laggancio_alla_libreria() {
    let hash = hash_password("prova").unwrap();
    assert!(verify_password(&hash, "prova"));
}
