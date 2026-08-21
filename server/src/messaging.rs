use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn send_message(
    writer: &mut OwnedWriteHalf,
    msg: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = serde_json::to_string(msg)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn receive_message(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Option<ClientMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = String::new();
    let bytes_read = reader.read_line(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(None); // Connessione chiusa dal client 
    }
    let msg = serde_json::from_str::<ClientMessage>(buffer.trim())?;
    Ok(Some(msg))
}

// pub async fn send_message_broadcast()
