use crate::input::read_line;
use crate::messaging::{receive_message, send_message};
use common::protocol::{AuthAction, ClientMessage, ServerMessage};
use tokio::io::BufReader;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn authenticate(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let action = choose_auth_action().await?;
    match action {
        AuthAction::Login => login(reader, writer).await,
        AuthAction::Register => register(reader, writer).await,
    }
}

async fn choose_auth_action() -> Result<AuthAction, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        println!("\nScegli un'opzione:");
        println!("1. Login");
        println!("2. Registrazione");
        let choice = read_line("Inserisci la tua scelta (1 o 2): ").await?;
        match choice.as_str() {
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
        let mut username;
        let mut password;

        loop {
            username = read_line("Inserisci username: ").await?;

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

        // 3. Invio messaggio di Login
        let login_msg = ClientMessage::Login { username, password };
        send_message(writer, &login_msg).await?;

        // 4. Attesa della risposta del server
         let server_msg = match receive_message(reader).await {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                eprintln!("\n[AUTH] Connessione chiusa dal server.");
                return Ok(false);
            }
            Err(e) => {
                eprintln!("\n[AUTH] Errore di comunicazione con il server: {}", e);
                return Ok(false);
            }
        };

       match server_msg {
            ServerMessage::AuthResult { success, reason } => {
                if success {
                    println!("\n[AUTH] Autenticazione riuscita!");
                    if let Ok(Some(ServerMessage::DirectMessage { message })) =
                        receive_message(reader).await
                    {
                        println!("{}", message);
                    }
                    return Ok(true);
                } else {
                    let msg = reason.unwrap_or_else(|| "Credenziali non valide".to_string());
                    eprintln!("\n[AUTH] Autenticazione fallita: {}", msg);
                    continue;
                }
            }
            ServerMessage::BroadcastMessage { message } => {
                println!("\n[SERVER]: {}", message);
                return Ok(false);
            }
            ServerMessage::Error { message } => {
                eprintln!("\n[AUTH] Errore dal server: {}", message);
                return Ok(false);
            }
            _ => {
                eprintln!("\n[AUTH] Risposta inattesa dal server durante il login.");
                return Ok(false);
            }
        }
    }
}

async fn register(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut username;
    let mut password;
    let mut password1;

    // 1. Richiesta username
    loop {
        username = read_line("Inserisci username: ").await?;

        if username.is_empty() {
            eprintln!("Lo username non può essere vuoto.");
            continue;
        }
        break;
    }

    // 2. Richiesta password (mascherata) 2 volte
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

    // 3. Invio messaggio di Register
    let register_msg = ClientMessage::Register { username, password };
    send_message(writer, &register_msg).await?;

    // 4. Attesa della risposta del server
    match receive_message(reader).await? {
        Some(ServerMessage::AuthResult { success, reason }) => {
            if success {
                println!("\n[AUTH] Registrazione riuscita!");
                let authenticated = login(reader, writer).await?;

                if authenticated {
                    println!("\n[AUTH] Autenticazione riuscita dopo la registrazione!");
                } else {
                    eprintln!("\n[AUTH] Autenticazione fallita dopo la registrazione.");
                    return Ok(false);
                }
                Ok(true)
            } else {
                let msg = reason.unwrap_or_else(|| "Registrazione fallita".to_string());
                eprintln!("\n[AUTH] Registrazione fallita: {}", msg);
                Ok(false)
            }
        }
        Some(msg) => {
            eprintln!(
                "\n[AUTH] Risposta inattesa dal server durante la registrazione: {:?}",
                msg
            );
            Ok(false)
        }
        None => {
            eprintln!("\n[AUTH] Connessione chiusa dal server durante la registrazione.");
            Ok(false)
        }
    }
}

