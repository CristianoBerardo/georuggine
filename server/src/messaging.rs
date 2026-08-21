use common::protocol::ServerMessage;
use tokio::io::AsyncWriteExt;

pub async fn send_message(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    msg: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = serde_json::to_string(msg)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
