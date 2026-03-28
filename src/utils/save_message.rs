use sqlx::SqlitePool;

pub async fn save_message(
    pool: &SqlitePool,
    user_id: i64,
    content: String,
    is_bot_message: bool,
    message_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO Conversation (user_id, message, message_type, is_bot_message)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(user_id)
    .bind(content)
    .bind(message_type)
    .bind(is_bot_message)
    .execute(pool)
    .await?;

    Ok(())
}
