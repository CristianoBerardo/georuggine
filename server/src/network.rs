use crate::handlers::handle_connection::handle_connection;
use crate::state::AppState;
use tokio::sync::oneshot;

// Prende in input l'indirizzo su cui mettersi in ascolto, lo stato condiviso
// dell'applicazione, e un canale oneshot su cui segnalare quando il server è
// davvero in ascolto
pub async fn run_server(
    addr: &str,
    state: AppState,
    ready_tx: oneshot::Sender<()>,
) -> std::io::Result<()> {
    // Crea un listener TCP che ascolta le connessioni in arrivo sull'indirizzo
    // in input
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server in ascolto su {}", addr);
    let _ = ready_tx.send(());

    loop {
        let (socket, _) = listener.accept().await?;

        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, state).await {
                eprintln!("Errore nella gestione della connessione: {}", e);
            }
        });
    }
}
