use crate::state::{AppState, Info, UserState, Username};

use crate::db::get_all_users;

pub async fn init_status_map(
    state: &mut AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let users = get_all_users(&state.db).await?;
    for user in users {
        state.user_status.write().await.insert(
            user,
            Info {
                state: UserState::Sconnesso,
                s: 0,
            },
        );
    }
    Ok(())
}
