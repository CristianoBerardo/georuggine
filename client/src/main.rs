use crate::movement_sim::movement_sim;
use crate::tools::read_movement_data::read_movement_data;
use common::protocol::{ClientMessage, ServerMessage};
use listener::listen;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;

mod auth;
mod input;
mod listener;
mod menu;
mod messaging;
mod movement_sim;
mod tools;

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
    let (tx, mut rx) = channel::<ClientMessage>(100);

    // Canale per inoltrare le risposte di autenticazione dal listener ad auth
    let (auth_resp_tx, mut auth_resp_rx) = channel::<ServerMessage>(10);

    // Flag condiviso: indica al listener se il menu è attivo
    let menu_active = Arc::new(AtomicBool::new(false));

    // 3. Task dedicato alla scrittura: riceve da rx e chiama `send_message` su writer
    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = messaging::send_message(&mut writer, &msg).await {
                eprintln!("[WRITER] Errore nell'invio del messaggio: {}", e);
                break;
            }
        }
    });

    // 4. Avvio immediato del listener per i messaggi asincroni dal server
    let mut listener_handle = tokio::spawn(listen(reader, auth_resp_tx, menu_active.clone()));

    // 5. Login o registrazione
    let authenticated = auth::authenticate(&tx, &mut auth_resp_rx).await?;
    if !authenticated {
        println!("\nOperazione completata. Disconnessione.");
        listener_handle.abort();
        drop(tx);
        let _ = writer_handle.await;
        return Ok(());
    }

    // 6. Avvio della simulazione del movimento
    let movement_handle = tokio::spawn(movement_sim(positions.clone(), tx.clone()));
    // Menu principale
    let mut menu_handle = tokio::spawn(menu::menu(tx.clone(), menu_active.clone()));

    // Se il server chiude la connessione (listener termina) OPPURE l'utente esce dal menu
    tokio::select! {
        _ = &mut listener_handle => {
            menu_handle.abort();
        }
        res = &mut menu_handle => {
            if let Ok(Err(e)) = res {
                eprintln!("[CLIENT] Errore nel menu: {}", e);
            }
            listener_handle.abort();
        }
    }
    // Pulizia e chiusura ordinata
    movement_handle.abort();

    // `tx` (il sender originale) e tutti i suoi clone rimasti attivi negli
    // altri task vanno droppati prima di aspettare `writer_handle`
    drop(tx);
    let _ = writer_handle.await;

    Ok(())
}
