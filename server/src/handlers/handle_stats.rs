use crate::db::{get_track_points_by_user_id_in_period, get_user_by_username};
use crate::messaging::send_message;
use crate::state::AppState;
use crate::stats::compute_stats;
use common::protocol::{ServerMessage, TimePeriod};
use tokio::net::tcp::OwnedWriteHalf;

pub async fn handle_stats(
    state: &AppState,
    writer: &mut OwnedWriteHalf,
    username: &str,
    period: TimePeriod,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Recupero l'utente dal database
    let user = match get_user_by_username(&state.db, username).await? {
        Some(u) => u,
        None => {
            let err_msg = ServerMessage::Error {
                message: format!("Utente '{}' non trovato", username),
            };
            send_message(writer, &err_msg).await?;
            return Ok(());
        }
    };

    // 2. Recupero i punti di tracciamento per l'utente nel periodo specificato
    let points =
        get_track_points_by_user_id_in_period(&state.db, user.id.unwrap(), &period).await?;

    // 3. Calcolo le statistiche
    let stats = compute_stats(&points);

    // 4. Invio le statistiche al client
    let result = ServerMessage::StatsResult {
        stats,
        timestamp: chrono::Utc::now(),
    };
    send_message(writer, &result).await?;

    Ok(())
}
