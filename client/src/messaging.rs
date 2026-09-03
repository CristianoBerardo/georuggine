use common::protocol::ClientMessage;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

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
