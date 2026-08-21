use crate::handlers::handle_connection::handle_connection;
use crate::state::AppState;

// Prende in input l'indirizzo su cui mettersi in ascolto e lo stato condiviso dell'applicazione
pub async fn run_server(addr: &str, state: AppState) -> std::io::Result<()> {
    // Crea un listener TCP che ascolta le connessioni in arrivo sull'indirizzo in input
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server in ascolto su {}", addr);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("Nuova connessione da {}", peer_addr);
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, state).await {
                eprintln!("Errore nella gestione della connessione: {}", e);
            }
        });
    }
}
