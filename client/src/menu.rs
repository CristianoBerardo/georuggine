use crate::input::read_line;
use common::protocol::{ClientMessage, TimePeriod};
use std::io::{self, Write};
use tokio::sync::mpsc::Sender;

/// Stampa il menu
pub fn print_menu() {
    println!("\n=== Menu Principale ===");
    println!("1. Invia messaggio al server");
    println!("2. Stampa statistiche");
    println!("3. Disconnetti");
    print!("Scelta: ");
    let _ = io::stdout().flush();
}

async fn choose_period() -> Result<TimePeriod, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        println!("Periodo:");
        println!("1. Oggi");
        println!("2. Questa settimana");
        println!("3. Questo mese");
        let choice = read_line("Scelta periodo: ").await?;

        match choice.as_str() {
            "1" => return Ok(TimePeriod::Today),
            "2" => return Ok(TimePeriod::ThisWeek),
            "3" => return Ok(TimePeriod::ThisMonth),
            _ => {
                eprintln!("Scelta non valida. Riprova.");
                continue;
            }
        }
    }
}

/// Mostra il menu e resta in loop finché l'utente non sceglie di disconnettersi
pub async fn menu(
    tx: Sender<ClientMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_menu();

    loop {
        let choice = read_line("").await?; // Prompt "Scelta: " già stampato da print_menu()

        match choice.as_str() {
            // Invia messaggio al server
            "1" => {
                let message = read_line("Messaggio: ").await?;
                tx.send(ClientMessage::ChatMessage { message }).await?;
                print_menu(); // Stampato qui poiché non si prevedono messaggi di risposta dal server per questa azione
            }
            // Stampa statistiche
            "2" => {
                let period = choose_period().await?;
                tx.send(ClientMessage::QueryStats { period }).await?;
                println!("Richiesta inviata, in attesa della risposta dal server...");
            }
            // Disconnetti
            "3" => {
                println!("Disconnessione in corso...");
                break;
            }
            _ => {
                eprintln!("Scelta non valida. Riprova.");
                print_menu();
            }
        }
    }

    Ok(())
}
