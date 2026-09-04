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

pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT username FROM users")
        .fetch_all(pool)
        .await
}

pub async fn insert_user(
    pool: &SqlitePool,
    username: String,
    password_hash: String,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&username)
        .bind(&password_hash)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_track_point(pool: &SqlitePool, tp: &TrackPoint) -> Result<i64, sqlx::Error> {
    let result =
        sqlx::query("INSERT INTO track_points (user_id, lat, lon, timestamp) VALUES (?, ?, ?, ?)")
            .bind(tp.user_id)
            .bind(tp.lat)
            .bind(tp.lon)
            .bind(tp.timestamp)
            .execute(pool)
            .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_track_points_by_user_id(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<TrackPoint>, sqlx::Error> {
    sqlx::query_as::<_, TrackPoint>(
        "SELECT id, user_id, lat, lon, timestamp FROM track_points
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
        "SELECT id, user_id, lat, lon, timestamp FROM track_points
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
        "SELECT id, user_id, lat, lon, timestamp FROM track_points
    WHERE user_id = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    // Database SQLite in memoria, isolato per ogni test: mai lo stesso file
    // usato dal server vero. `max_connections(1)` è necessario perché ogni
    // nuova connessione a "sqlite::memory:" vedrebbe altrimenti un database
    // vuoto e diverso, invece di condividere quello appena creato.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE track_points (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                lat REAL NOT NULL,
                lon REAL NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn track_point(
        user_id: i64,
        lat: f64,
        lon: f64,
        timestamp: chrono::DateTime<Utc>,
    ) -> TrackPoint {
        TrackPoint {
            id: None,
            user_id,
            lat,
            lon,
            timestamp,
        }
    }

    #[tokio::test]
    async fn utente_inserito_si_ritrova_per_username() {
        let pool = test_pool().await;
        insert_user(&pool, "mario".to_string(), "hash123".to_string())
            .await
            .unwrap();

        let user = get_user_by_username(&pool, "mario")
            .await
            .unwrap()
            .expect("l'utente dovrebbe esistere");
        assert_eq!(user.username, "mario");
        assert_eq!(user.password_hash, "hash123");
    }

    #[tokio::test]
    async fn utente_inesistente_restituisce_none() {
        let pool = test_pool().await;
        let user = get_user_by_username(&pool, "fantasma").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn get_all_users_elenca_tutti_gli_username() {
        let pool = test_pool().await;
        insert_user(&pool, "mario".to_string(), "h1".to_string())
            .await
            .unwrap();
        insert_user(&pool, "anna".to_string(), "h2".to_string())
            .await
            .unwrap();

        let mut users = get_all_users(&pool).await.unwrap();
        users.sort();
        assert_eq!(users, vec!["anna".to_string(), "mario".to_string()]);
    }

    #[tokio::test]
    async fn punto_di_tracciamento_si_ritrova_per_utente() {
        let pool = test_pool().await;
        let user_id = insert_user(&pool, "mario".to_string(), "hash".to_string())
            .await
            .unwrap();

        let tp = track_point(user_id, 45.0, 9.0, Utc::now());
        insert_track_point(&pool, &tp).await.unwrap();

        let points = get_track_points_by_user_id(&pool, user_id)
            .await
            .unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].lat, 45.0);
        assert_eq!(points[0].lon, 9.0);
    }

    #[tokio::test]
    async fn punti_ordinati_per_timestamp_crescente() {
        let pool = test_pool().await;
        let user_id = insert_user(&pool, "mario".to_string(), "hash".to_string())
            .await
            .unwrap();

        let now = Utc::now();
        insert_track_point(
            &pool,
            &track_point(user_id, 45.1, 9.1, now + chrono::Duration::seconds(30)),
        )
        .await
        .unwrap();
        insert_track_point(&pool, &track_point(user_id, 45.0, 9.0, now))
            .await
            .unwrap();

        let points = get_track_points_by_user_id(&pool, user_id)
            .await
            .unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].lat, 45.0); // il più vecchio prima
        assert_eq!(points[1].lat, 45.1);
    }

    #[tokio::test]
    async fn filtro_per_periodo_esclude_punti_troppo_vecchi() {
        let pool = test_pool().await;
        let user_id = insert_user(&pool, "mario".to_string(), "hash".to_string())
            .await
            .unwrap();

        let now = Utc::now();
        let vecchio = now - chrono::Duration::days(40); // fuori da "questo mese"
        insert_track_point(&pool, &track_point(user_id, 45.0, 9.0, vecchio))
            .await
            .unwrap();
        insert_track_point(&pool, &track_point(user_id, 45.1, 9.1, now))
            .await
            .unwrap();

        let points = get_track_points_by_user_id_in_period(&pool, user_id, &TimePeriod::ThisMonth)
            .await
            .unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].lat, 45.1);
    }

    #[tokio::test]
    async fn ultimo_punto_e_quello_piu_recente() {
        let pool = test_pool().await;
        let user_id = insert_user(&pool, "mario".to_string(), "hash".to_string())
            .await
            .unwrap();

        let now = Utc::now();
        insert_track_point(&pool, &track_point(user_id, 45.0, 9.0, now))
            .await
            .unwrap();
        insert_track_point(
            &pool,
            &track_point(user_id, 45.1, 9.1, now + chrono::Duration::seconds(30)),
        )
        .await
        .unwrap();

        let last = get_last_track_point_by_user_id(&pool, user_id)
            .await
            .unwrap()
            .expect("dovrebbe esserci un ultimo punto");
        assert_eq!(last.lat, 45.1);
    }

    #[tokio::test]
    async fn nessun_punto_restituisce_none_per_ultimo_punto() {
        let pool = test_pool().await;
        let user_id = insert_user(&pool, "mario".to_string(), "hash".to_string())
            .await
            .unwrap();

        let last = get_last_track_point_by_user_id(&pool, user_id)
            .await
            .unwrap();
        assert!(last.is_none());
    }
}
