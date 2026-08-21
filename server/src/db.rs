use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::Utc;
use common::{
    models::{TrackPoint, User},
    protocol::TimePeriod,
};
use sqlx::SqlitePool;

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn insert_user(pool: &SqlitePool, user: &User) -> Result<i64, sqlx::Error> {
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&user.username)
        .bind(&user.password_hash)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_track_point(pool: &SqlitePool, tp: &TrackPoint) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO track_points (user_id, lat, lon, timestamp, state) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(tp.user_id)
    .bind(tp.lat)
    .bind(tp.lon)
    .bind(tp.timestamp)
    .bind(&tp.state)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_track_points_by_user_id(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<TrackPoint>, sqlx::Error> {
    sqlx::query_as::<_, TrackPoint>(
        "SELECT id, user_id, lat, lon, timestamp, state FROM track_points
    WHERE user_id = ? ORDER BY timestamp ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_track_points_by_user_id_in_period(
    pool: &SqlitePool,
    user_id: i64,
    period: &TimePeriod,
) -> Result<Vec<TrackPoint>, sqlx::Error> {
    let since = match period {
        TimePeriod::Today => Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        TimePeriod::ThisWeek => Utc::now() - chrono::Duration::days(7),
        TimePeriod::ThisMonth => Utc::now() - chrono::Duration::days(30),
    };

    sqlx::query_as::<_, TrackPoint>(
        "SELECT id, user_id, lat, lon, timestamp, state FROM track_points
    WHERE user_id = ? AND timestamp >= ? ORDER BY timestamp ASC",
    )
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await
}

pub async fn get_last_track_point_by_user_id(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Option<TrackPoint>, sqlx::Error> {
    sqlx::query_as::<_, TrackPoint>(
        "SELECT id, user_id, lat, lon, timestamp, state FROM track_points
    WHERE user_id = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }