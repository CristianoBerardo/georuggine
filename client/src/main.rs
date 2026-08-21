use common::protocol::{ClientMessage, ServerMessage};
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GeoRuggine Client CLI ===");

    // 1. Richiesta username
    print!("Inserisci username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    if username.is_empty() {
        eprintln!("Lo username non può essere vuoto.");
        return Ok(());
    }

    // 2. Richiesta password (mascherata)
    let password = rpassword::prompt_password("Inserisci password: ")?;
    if password.is_empty() {
        eprintln!("La password non può essere vuota.");
        return Ok(());
    }

    // 3. Connessione al server TCP
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

    // 4. Invia messaggio di Login (serializzato con serde in formato JSON terminato da \n)
    let login_msg = ClientMessage::Login { username, password };
    let mut payload = serde_json::to_string(&login_msg)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;

    // 5. Lettura delle risposte dal server
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        match serde_json::from_str::<ServerMessage>(trimmed) {
            Ok(server_msg) => match server_msg {
                ServerMessage::AuthResult { success, reason } => {
                    if success {
                        println!("[AUTH] Autenticazione riuscita!");
                    } else {
                        let msg = reason.unwrap_or_else(|| "Credenziali non valide".to_string());
                        eprintln!("[AUTH] Autenticazione fallita: {}", msg);
                        break;
                    }
                }
                ServerMessage::DirectMessage { message } => {
                    println!("[RISPOSTA QUERY DI PROVA]: {}", message);
                    // Ricevuto il messaggio con i dati dal server, il client one-shot ha completato l'operazione
                    break;
                }
                ServerMessage::BroadcastMessage { message } => {
                    println!("[BROADCAST]: {}", message);
                }
                ServerMessage::StatsResult { stats } => {
                    println!("[STATISTICHE]: {:?}", stats);
                    break;
                }
                ServerMessage::Error { message } => {
                    eprintln!("[ERRORE SERVER]: {}", message);
                    break;
                }
            },
            Err(e) => {
                eprintln!("Errore nella deserializzazione del messaggio dal server: {}", e);
                break;
            }
        }
        line.clear();
    }

    println!("\nOperazione completata. Disconnessione.");
    Ok(())
}
