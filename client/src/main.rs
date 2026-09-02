use crate::movement_sim::movement_sim;
use crate::tools::read_movement_data::read_movement_data;
use common::protocol::{ClientMessage, ServerMessage};
use listener::listen;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;

mod auth;
mod console;
mod input;
mod listener;
mod menu;
mod messaging;
mod movement_sim;
mod tools;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== GeoRuggine Client CLI ===");

    let positions = read_movement_data("client/src/movement_data/torino-asti.csv")?;

    // 1. Connessione al server TCP
    println!("\nConnessione a 127.0.0.1:8080 in corso...");
    let stream = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(s) => {
            println!("Connessione stabilita con successo!");
            s
        }
        Err(e) => {
            eprintln!("Impossibile connettersi al server: {}", e);
            return Ok(());
        }
    };

    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);

    // 2. Canale MPSC per inviare messaggi al server da vari tasks
    let (client_msg_tx, mut client_msg_rx) = channel::<ClientMessage>(100);

    // Canale per inoltrare i messaggi del server ad auth
    let (server_msg_tx, mut server_msg_rx) = channel::<ServerMessage>(100);

    // 3. Task dedicato alla scrittura: riceve da rx e chiama send_message su writer
    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = client_msg_rx.recv().await {
            if let Err(e) = messaging::send_message(&mut writer, &msg).await {
                eprintln!("[WRITER] Errore nell'invio del messaggio: {}", e);
                break;
            }
        }
    });

    // 4. Avvio immediato del listener per i messaggi asincroni dal server
    let mut listener_handle = tokio::spawn(listen(reader, server_msg_tx));

    // 5. Login o registrazione
    let Some(username) = auth::authenticate(&client_msg_tx, &mut server_msg_rx).await? else {
        println!("\nOperazione completata. Disconnessione.");
        listener_handle.abort();
        drop(client_msg_tx);
        let _ = writer_handle.await;
        return Ok(());
    };

    // 6. Avvio della simulazione del movimento
    let movement_handle = tokio::spawn(movement_sim(positions.clone(), client_msg_tx.clone()));
    // Menu principale
    ui::main_ui::run(username, &client_msg_tx, &mut server_msg_rx).await?;
    // Pulizia e chiusura ordinata
    movement_handle.abort();

    // `tx` (il sender originale) e tutti i suoi clone rimasti attivi negli
    // altri task vanno droppati prima di aspettare `writer_handle`
    drop(client_msg_tx);
    let _ = writer_handle.await;

    Ok(())
}
