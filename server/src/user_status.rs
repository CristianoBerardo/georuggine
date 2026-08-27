use crate::state::{AppState, Info, UserStatus, Username};

use crate::db::get_all_users;

pub async fn init_status_map(
    state: &mut AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let users = get_all_users(&state.db).await?;
    for user in users {
        state.user_status.write().await.insert(
            user,
            Info {
                status: UserStatus::Sconnesso,
                s: 0,
            },
        );
    }
    Ok(())
}

pub async fn update_user_status(
    state: &AppState,
    username: &Username,
    new_status: UserStatus,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    if let Some(info) = user_status.get_mut(username) {
        info.status = new_status;
    } else {
        eprintln!("Utente {} non trovato nella mappa degli stati.", username);
    }
    Ok(())
}

pub async fn update_user_seconds(
    state: &AppState,
    username: &Username,
    seconds: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    if let Some(info) = user_status.get_mut(username) {
        info.s = seconds;
    } else {
        eprintln!("Utente {} non trovato nella mappa degli stati.", username);
    }
    Ok(())
}

pub async fn get_user_info(
    state: &AppState,
    username: &Username,
) -> Result<Option<Info>, Box<dyn std::error::Error + Send + Sync>> {
    let user_status = state.user_status.read().await;
    Ok(user_status.get(username).cloned())
}

pub async fn add_user_to_status_map(
    state: &AppState,
    username: Username,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut user_status = state.user_status.write().await;
    user_status.insert(
        username,
        Info {
            status: UserStatus::Sconnesso,
            s: 0,
        },
    );
    Ok(())
}
