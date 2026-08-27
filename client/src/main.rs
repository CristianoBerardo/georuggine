use crate::movement_sim::movement_sim;
use crate::tools::read_movement_data::read_movement_data;
use common::protocol::ClientMessage;
use listener::listen;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;

mod auth;
mod listener;
mod menu;
mod messaging;
mod movement_sim;
mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== GeoRuggine Client CLI ===");

    let positions = read_movement_data("client/src/movement_data/torino-asti.csv")?;
    println!(
        "Dati di movimento letti con successo: {:?} posizioni",
        positions
    );

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
    let mut reader = BufReader::new(reader);

    // 2. Login o registrazione
    let authenticated = auth::authenticate(&mut reader, &mut writer).await?;
    if !authenticated {
        println!("\nOperazione completata. Disconnessione.");
        return Ok(());
    }

    // 3. Avvio del listener per i messaggi asincroni dal server,
    // posso passare il possesso a listener che ascolta i messaggi in arrivo
    let listener_handle = tokio::spawn(listen(reader));

    // 4. Canale MPSC per inviare messaggi al server da vari tasks
    let (tx, mut rx) = channel::<ClientMessage>(100);

    // 5. Task dedicato alla scrittura: riceve da rx e chiama `send_message` su writer
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = messaging::send_message(&mut writer, &msg).await {
                eprintln!("[WRITER] Errore nell'invio del messaggio: {}", e);
                continue;
            }
        }
    });

    // Avvio della simulazione del movimento
    tokio::spawn(movement_sim(positions.clone(), tx.clone()));

    // Menu principale (può usare `tx` per inviare messaggi al server)
    // es: menu::menu(tx.clone()).await?; e dentro al menù tx.send()

    listener_handle.await?;
    println!("\nOperazione completata. Disconnessione.");
    Ok(())
}
