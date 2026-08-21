use crate::handlers::handle_login::handle_login;
use crate::state::AppState;

use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::messaging::send_message;

pub async fn handle_connection(
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
                        handle_login(
                            username,
                            password,
                            &state,
                            &mut writer,
                            &tx,
                            &mut authenticated_user,
                        ).await?;
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
