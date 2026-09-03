use crate::db::{get_last_track_point_by_user_id, insert_track_point};
use crate::state::{AppState, UserStatus};
use crate::user_status::{get_user_info, update_user_seconds, update_user_status};
use common::models::{Position, TrackPoint, User};

const MAX_SEC: u64 = 180; // 3 minuti

pub async fn handle_new_track_point(
    state: &AppState,
    authenticated_user: &User,
    position: Position,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = authenticated_user
        .id
        .expect("un utente autenticato deve avere un id valido nel DB");

    let last_track_point = get_last_track_point_by_user_id(&state.db, user_id).await?;

    let info = get_user_info(state, &authenticated_user.username)
        .await?
        .ok_or_else(|| {
            format!(
                "Informazioni sull'utente {} non trovate nella mappa degli stati.",
                authenticated_user.username
            )
        })?;

    let elapsed_time: u64 = if let Some(last_tp) = &last_track_point {
        (position.timestamp - last_tp.timestamp)
            .num_seconds()
            .max(0) as u64
    } else {
        0
    };

    if last_track_point.is_none() {
        // Primo track point ricevuto per questo utente
        update_user_status(state, &authenticated_user.username, UserStatus::InMovimento).await?;
        update_user_seconds(state, &authenticated_user.username, 0).await?;
    } else {
        // SCONNESSO o PROBLEMA (recovery)
        if info.status == UserStatus::Sconnesso || info.status == UserStatus::Problema {
            // Se l'utente era sconnesso, lo consideriamo in movimento al primo track point ricevuto
            update_user_status(state, &authenticated_user.username, UserStatus::InMovimento)
                .await?;
            update_user_seconds(state, &authenticated_user.username, 0).await?;
        }
        // IN MOVIMENTO
        else if info.status == UserStatus::InMovimento {
            if last_track_point.as_ref().unwrap().lat == position.lat
                && last_track_point.as_ref().unwrap().lon == position.lon
            {
                if info.s >= MAX_SEC {
                    // L'utente è fermo da più di 3 minuti
                    update_user_status(state, &authenticated_user.username, UserStatus::Fermo)
                        .await?;
                } else {
                    // L'utente è fermo da meno di 3 minuti
                    update_user_seconds(state, &authenticated_user.username, info.s + elapsed_time)
                        .await?;
                }
            } else {
                // L'utente continua a muoversi
                update_user_status(state, &authenticated_user.username, UserStatus::InMovimento)
                    .await?;
                update_user_seconds(state, &authenticated_user.username, 0).await?;
            }
        // FERMO
        } else {
            if last_track_point.as_ref().unwrap().lat != position.lat
                || last_track_point.as_ref().unwrap().lon != position.lon
            {
                // L'utente ha iniziato a muoversi
                update_user_status(state, &authenticated_user.username, UserStatus::InMovimento)
                    .await?;
                update_user_seconds(state, &authenticated_user.username, 0).await?;
            } else {
                // L'utente è ancora fermo
                update_user_seconds(state, &authenticated_user.username, info.s + elapsed_time)
                    .await?;
            }
        }
    }

    let new_track_point = TrackPoint {
        id: None, // L'ID sarà generato automaticamente dal database
        user_id,
        lat: position.lat,
        lon: position.lon,
        timestamp: position.timestamp,
    };

    // Inserisci il nuovo track point nel database
    insert_track_point(&state.db, &new_track_point).await?;
    Ok(())
}
