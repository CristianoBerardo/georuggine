use crate::handlers::handle_connection::handle_connection;
use crate::state::AppState;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot::Sender;

// Prende in input l'indirizzo su cui mettersi in ascolto, lo stato condiviso
// dell'applicazione, e un canale oneshot su cui segnalare quando il server è
// davvero in ascolto
pub async fn run_server(
    addr: &str,
    state: AppState,
    ready_tx: Sender<SocketAddr>, // utile per sapere l'indirizzo su cui il server è in ascolto per i test
) -> std::io::Result<()> {
    // Crea un listener TCP che ascolta le connessioni in arrivo sull'indirizzo
    // in input
    let listener = TcpListener::bind(addr).await?;
    println!("Server in ascolto su {}", addr);
    let _ = ready_tx.send(listener.local_addr()?);

    loop {
        let (socket, _) = listener.accept().await?;

        let state = state.clone();
        let error_tx = state.error_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, state).await {
                let message = format!("Errore nella gestione della connessione: {}", e);
                let _ = error_tx.send(crate::state::IncomingError {
                    message,
                    timestamp: chrono::Utc::now(),
                });
            }
        });
    }
}
