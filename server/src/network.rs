use crate::db::{get_user_by_username, verify_password};
use crate::state::AppState;
use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

// Prende in input l'indirizzo su cui mettersi in ascolto e lo stato condiviso dell'applicazione
pub async fn run_server(addr: &str, state: AppState) -> std::io::Result<()> {
    // Crea un listener TCP che ascolta le connessioni in arrivo sull'indirizzo in input
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server in ascolto su {}", addr);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("Nuova connessione da {}", peer_addr);
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, state).await {
                eprintln!("Errore nella gestione della connessione: {}", e);
            }
        });
    }
}

async fn handle_connection(
    socket: TcpStream,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let mut authenticated_user: Option<String> = None;
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    loop {
        line.clear();
        tokio::select! {
            // Ricezione messaggi dal client
            bytes_read = reader.read_line(&mut line) => {
                let bytes = bytes_read?;
                if bytes == 0 {
                    // Connessione chiusa dal client
                    break;
                }

                let client_msg: ClientMessage = match serde_json::from_str(line.trim()) {
                    Ok(msg) => msg,
                    Err(e) => {
                        let err_msg = ServerMessage::Error {
                            message: format!("Formato messaggio non valido: {}", e),
                        };
                        send_message(&mut writer, &err_msg).await?;
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::Login { username, password } => {
                        println!("Tentativo di login per l'utente: {}", username);
                        match get_user_by_username(&state.db, &username).await {
                            Ok(Some(user)) if verify_password(&user.password_hash, &password) => {
                                println!("Utente {} autenticato con successo!", username);
                                authenticated_user = Some(username.clone());

                                // Registra il canale nella mappa delle connessioni
                                {
                                    let mut conns = state.connections.write().await;
                                    conns.insert(username.clone(), tx.clone());
                                }

                                // Invia conferma di autenticazione
                                let auth_ok = ServerMessage::AuthResult {
                                    success: true,
                                    reason: None,
                                };
                                send_message(&mut writer, &auth_ok).await?;

                                // Query di prova / messaggio di benvenuto con i dati dell'utente dal DB
                                let test_query_msg = ServerMessage::DirectMessage {
                                    message: format!(
                                        "Benvenuto {}, ID utente: {:?}. Query di prova su DB eseguita con successo!",
                                        user.username, user.id
                                    ),
                                };
                                send_message(&mut writer, &test_query_msg).await?;
                            }
                            Ok(_) => {
                                println!("Autenticazione fallita per l'utente: {}", username);
                                let auth_err = ServerMessage::AuthResult {
                                    success: false,
                                    reason: Some("Credenziali non valide".to_string()),
                                };
                                send_message(&mut writer, &auth_err).await?;
                            }
                            Err(e) => {
                                eprintln!("Errore DB durante login di {}: {}", username, e);
                                let err_resp = ServerMessage::Error {
                                    message: format!("Errore interno del server durante il recupero utente: {}", e),
                                };
                                send_message(&mut writer, &err_resp).await?;
                            }
                        }
                    }
                    ClientMessage::Register { .. } => {
                        let err_msg = ServerMessage::Error {
                            message: "Registrazione non ancora implementata".to_string(),
                        };
                        send_message(&mut writer, &err_msg).await?;
                    }
                    ClientMessage::PositionUpdate { .. } => {}
                    ClientMessage::ChatMessage { message } => {
                        println!("Messaggio ricevuto da {:?}: {}", authenticated_user, message);
                    }
                    ClientMessage::QueryStats { .. } => {}
                }
            }
            // Invio messaggi accodati nel canale mpsc verso il client
            Some(outgoing_msg) = rx.recv() => {
                send_message(&mut writer, &outgoing_msg).await?;
            }
        }
    }

    // Pulizia connessione quando il client si disconnette
    if let Some(user) = authenticated_user {
        let mut conns = state.connections.write().await;
        conns.remove(&user);
        println!("Utente {} disconnesso.", user);
    }

    Ok(())
}

async fn send_message(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    msg: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = serde_json::to_string(msg)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
