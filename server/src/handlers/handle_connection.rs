use crate::handlers::{handle_login::handle_login, handle_registration::handle_registration};
use crate::state::AppState;

use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::messaging::{receive_message, send_message};

pub async fn handle_connection(
    socket: TcpStream,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut authenticated_user: Option<String> = None;
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    loop {
        tokio::select! {
            // Ricezione messaggi dal client
            result = receive_message(&mut reader) => {
                let client_msg = match result {
                    Ok(None) => break, // Connessione chiusa dal client
                    Ok(Some(msg)) => msg,
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
                    ClientMessage::Register { username, password } => {
                        handle_registration(
                            username,
                            password,
                            &state,
                            &mut writer,
                            &tx,
                            &mut authenticated_user,
                        ).await?;
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
