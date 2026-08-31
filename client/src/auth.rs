use crate::input::read_line;
use common::protocol::{AuthAction, ClientMessage, ServerMessage};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn authenticate(
    tx: &Sender<ClientMessage>,
    rx: &mut Receiver<ServerMessage>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let action = choose_auth_action().await?;
    match action {
        AuthAction::Login => login(tx, rx).await,
        AuthAction::Register => register(tx, rx).await,
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
    tx: &Sender<ClientMessage>,
    rx: &mut Receiver<ServerMessage>,
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
            password =
                tokio::task::spawn_blocking(|| rpassword::prompt_password("Inserisci password: "))
                    .await
                    .unwrap()?;
            if password.is_empty() {
                eprintln!("La password non può essere vuota.");
                continue;
            }
            break;
        }

        // 3. Invio messaggio di Login
        let login_msg = ClientMessage::Login { username, password };
        if tx.send(login_msg).await.is_err() {
            eprintln!("\n[AUTH] Impossibile inviare il messaggio: canale di scrittura chiuso.");
            return Ok(false);
        }

        // 4. Attesa della risposta del server dal listener
        let server_msg = match rx.recv().await {
            Some(msg) => msg,
            None => {
                eprintln!("\n[AUTH] Connessione chiusa dal server.");
                return Ok(false);
            }
        };

        match server_msg {
            ServerMessage::AuthResult { success, reason } => {
                if success {
                    println!("\n[AUTH] Autenticazione riuscita!");
                    return Ok(true);
                } else {
                    let msg = reason.unwrap_or_else(|| "Credenziali non valide".to_string());
                    eprintln!("\n[AUTH] Autenticazione fallita: {}", msg);
                    continue;
                }
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
    tx: &Sender<ClientMessage>,
    rx: &mut Receiver<ServerMessage>,
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
        password =
            tokio::task::spawn_blocking(|| rpassword::prompt_password("Inserisci password: "))
                .await
                .unwrap()?;
        if password.is_empty() {
            eprintln!("La password non può essere vuota.");
            continue;
        }
        password1 =
            tokio::task::spawn_blocking(|| rpassword::prompt_password("Conferma password: "))
                .await
                .unwrap()?;
        if password != password1 {
            eprintln!("Le password non coincidono. Reinserire la password.");
            continue;
        }
        break;
    }

    // 3. Invio messaggio di Register
    let register_msg = ClientMessage::Register { username, password };
    if tx.send(register_msg).await.is_err() {
        eprintln!("\n[AUTH] Impossibile inviare il messaggio: canale di scrittura chiuso.");
        return Ok(false);
    }

    // 4. Attesa della risposta del server dal listener
    match rx.recv().await {
        Some(ServerMessage::AuthResult { success, reason }) => {
            if success {
                println!("\n[AUTH] Registrazione riuscita!");
                let authenticated = login(tx, rx).await?;

                if authenticated {
                    println!("\n[AUTH] Autenticazione riuscita dopo la registrazione!");
                    Ok(true)
                } else {
                    eprintln!("\n[AUTH] Autenticazione fallita dopo la registrazione.");
                    Ok(false)
                }
            } else {
                let msg = reason.unwrap_or_else(|| "Registrazione fallita".to_string());
                eprintln!("\n[AUTH] Registrazione fallita: {}", msg);
                Ok(false)
            }
        }
        Some(ServerMessage::Error { message }) => {
            eprintln!(
                "\n[AUTH] Errore dal server durante la registrazione: {}",
                message
            );
            Ok(false)
        }
        _ => {
            eprintln!("\n[AUTH] Connessione chiusa o risposta inattesa durante la registrazione.");
            Ok(false)
        }
    }
}
