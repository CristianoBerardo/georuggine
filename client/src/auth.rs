use crate::messaging::{receive_message, send_message};
use common::protocol::{AuthAction, ClientMessage, ServerMessage};
use std::io::{self, Write};
use tokio::io::BufReader;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn authenticate(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let action = choose_auth_action()?;
    match action {
        AuthAction::Login => login(reader, writer).await,
        AuthAction::Register => register(reader, writer).await,
    }
}

fn choose_auth_action() -> Result<AuthAction, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        println!("Scegli un'opzione:");
        println!("1. Login");
        println!("2. Registrazione");
        print!("Inserisci la tua scelta (1 o 2): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => return Ok(AuthAction::Login),
            "2" => return Ok(AuthAction::Register),
            _ => {
                eprintln!("Scelta non valida. Riprova.");
                continue;
            }
        }
    }
}

async fn login(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Richiesta username
    loop {
        let mut username = String::new();
        let mut password;

        loop {
            print!("Inserisci username: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut username)?;
            username = username.trim().to_string();

            if username.is_empty() {
                eprintln!("Lo username non può essere vuoto.");
                continue;
            }
            break;
        }

        // 2. Richiesta password (mascherata)
        loop {
            password = rpassword::prompt_password("Inserisci password: ")?;
            if password.is_empty() {
                eprintln!("La password non può essere vuota.");
                continue;
            }
            break;
        }

        // 3. Invio messaggio di Login (serializzato con serde in formato JSON terminato da \n)
        let login_msg = ClientMessage::Login { username, password };
        send_message(writer, &login_msg).await?;

        // 4. Attesa della risposta del server
        let server_msg = receive_message(reader).await?;
        match server_msg {
            Some(ServerMessage::AuthResult { success, reason }) => {
                if success {
                    println!("[AUTH] Autenticazione riuscita!");
                    return Ok(true);
                } else {
                    let msg = reason.unwrap_or_else(|| "Credenziali non valide".to_string());
                    eprintln!("[AUTH] Autenticazione fallita: {}", msg);
                    continue;
                }
            }
            _ => {
                eprintln!("Risposta inattesa dal server durante il login.");
                return Ok(false);
            }
        }
    }
}

async fn register(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut username = String::new();
    let mut password;
    let mut password1;

    // 1. Richiesta username
    loop {
        print!("Inserisci username: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut username)?;
        username = username.trim().to_string();

        if username.is_empty() {
            eprintln!("Lo username non può essere vuoto.");
            continue;
        }
        break;
    }

    // 2. Richiesta password (mascherata)
    loop {
        password = rpassword::prompt_password("Inserisci password: ")?;
        if password.is_empty() {
            eprintln!("La password non può essere vuota.");
            continue;
        }
        password1 = rpassword::prompt_password("Conferma password: ")?;
        if password != password1 {
            eprintln!("Le password non coincidono. Reinserire la password.");
            continue;
        }
        break;
    }

    // 3. Invio messaggio di Register (serializzato con serde in formato JSON terminato da \n)
    let register_msg = ClientMessage::Register { username, password };
    send_message(writer, &register_msg).await?;

    // 4. Attesa della risposta del server
    match receive_message(reader).await? {
        Some(ServerMessage::AuthResult { success, reason }) => {
            if success {
                println!("[AUTH] Registrazione riuscita!");
                login(reader, writer).await?;
                Ok(true)
            } else {
                let msg = reason.unwrap_or_else(|| "Registrazione fallita".to_string());
                eprintln!("[AUTH] Registrazione fallita: {}", msg);
                Ok(false)
            }
        }
        Some(msg) => {
            eprintln!(
                "Risposta inattesa dal server durante la registrazione: {:?}",
                msg
            );
            Ok(false)
        }
        None => {
            eprintln!("Connessione chiusa dal server durante la registrazione.");
            Ok(false)
        }
    }
}
