use crate::db::get_user_by_username;
use crate::state::AppState;

// Prende in input l'indirizzo su cui mettersi in ascolto e lo stato condiviso dell'applicazione
pub async fn run_server(addr: &str, state: AppState) -> std::io::Result<()> {
    // Crea un listener TCP che ascolta le connessioni in arrivo sull'indirizzo in input
    let listener = tokio::net::TcpListener::bind(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let state = state.clone();
        println!("Eccoci qui");
        tokio::spawn(async move {
            handle_connection(socket, state).await;
        });
    }
}

async fn handle_connection(socket: tokio::net::TcpStream, state: AppState) {
    // Funzioni che fanno lìhandling dei messaggi
    println!("Stiamo cercando di far funzinoare la connessione");
    println!("{:?}", get_user_by_username(&state.db, "mario").await);
}
