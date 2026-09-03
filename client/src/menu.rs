use crate::console;
use crate::input::{read_choice, read_line};
use common::protocol::{ClientMessage, TimePeriod};
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::Sender;

// Stampa il menu
pub fn print_menu() {
    println!("\n=== Menu Principale CLIENT ===");
    println!("1. Invia messaggio al server");
    println!("2. Stampa statistiche");
    println!("3. Disconnetti");
    print!("Scelta: ");
    let _ = io::stdout().flush();
}

// Passa `text` al coordinatore della console. Se viene stampato subito
// (l'utente non stava scrivendo nulla) e il menu è attivo, ristampa anche il
// menu subito dopo.
pub fn print_or_queue_with_menu(text: String, menu_active: &AtomicBool) {
    if console::print_or_queue(text) && menu_active.load(Ordering::Relaxed) {
        print_menu();
    }
}

// Chiude una protezione avviata manualmente (protegge la console durante
// l'attesa delle statistiche), mostrando text per primo, prima di qualunque
// altro messaggio accodato nel frattempo, e ristampa il menu.
pub fn end_wait_with(text: String, menu_active: &AtomicBool) {
    console::end_typing_with(text);
    if menu_active.load(Ordering::Relaxed) {
        print_menu();
    }
}

async fn choose_period() -> Result<TimePeriod, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        println!("Periodo:");
        println!("1. Oggi");
        println!("2. Questa settimana");
        println!("3. Questo mese");
        // A differenza della scelta principale, questa scelta è già dentro a
        // un'azione avviata dall'utente: mentre sceglie il periodo non
        // vogliamo che un messaggio in arrivo la interrompa (usiamo read_line
        // e non read_choice).
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

// Mostra il menu e resta in loop finché l'utente non sceglie di disconnettersi.
// Il menu viene ristampato all'inizio di ogni iterazione (e, se necessario,
// anche da print_or_queue_with_menu`quando un messaggio arriva mentre si è
// fermi sul prompt "Scelta: ").
pub async fn menu(
    tx: Sender<ClientMessage>,
    menu_active: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    menu_active.store(true, Ordering::Relaxed);

    loop {
        print_menu();
        let choice = read_choice("").await?; // Prompt "Scelta: " già stampato da print_menu()

        match choice.as_str() {
            // Invia messaggio al server
            "1" => {
                let message = read_line("Messaggio: ").await?;
                tx.send(ClientMessage::ChatMessage {
                    message,
                    timestamp: chrono::Utc::now(),
                })
                .await?;
            }
            // Stampa statistiche
            "2" => {
                let period = choose_period().await?;
                // Protegge la console finché non arriva la risposta: i
                // messaggi che arrivano nel frattempo vengono accodati e
                // mostrati solo dopo le statistiche.
                console::begin_typing();
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
            }
        }
    }

    Ok(())
}
