use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn send_message(
    writer: &mut OwnedWriteHalf,
    msg: &ClientMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = serde_json::to_string(msg)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn receive_message(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Option<ServerMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line).await?;
    if bytes_read == 0 {
        return Ok(None);
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match serde_json::from_str(trimmed) {
        Ok(msg) => Ok(Some(msg)),
        Err(e) => Err(e.into()),
    }
}
