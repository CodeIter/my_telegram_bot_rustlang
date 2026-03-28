use sqlx::{Row, SqlitePool};
use teloxide::types::User;

pub async fn upsert_user_and_get_id(pool: &SqlitePool, user: &User) -> Result<i64, sqlx::Error> {
    let telegram_id = user.id.0 as i64;
    let username = user.username.as_deref();
    let first_name = Some(user.first_name.as_str());
    let last_name = user.last_name.as_deref();

    let row = sqlx::query(
        r#"
        INSERT INTO users (telegram_id, username, first_name, last_name)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(telegram_id) DO UPDATE SET
            username = excluded.username,
            first_name = excluded.first_name,
            last_name = excluded.last_name
        RETURNING id
        "#,
    )
    .bind(telegram_id)
    .bind(username)
    .bind(first_name)
    .bind(last_name)
    .fetch_one(pool)
    .await?;

    let internal_id: i64 = row.get(0);
    Ok(internal_id)
}
