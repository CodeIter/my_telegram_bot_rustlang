use sqlx::{Row, SqlitePool};

/// A single entry from the Conversation table.
pub struct ConversationMessage {
    pub is_bot_message: bool,
    pub message: String,
}

/// Fetches the last `limit` text messages for `internal_user_id`,
/// returned in **oldest-first** order (ready to append as context).
///
/// Only `message_type = 'text'` rows are included so media placeholders
/// (photo, video, …) are not fed to the LLM.
pub async fn get_conversation_history(
    pool: &SqlitePool,
    internal_user_id: i64,
    limit: i64,
) -> Result<Vec<ConversationMessage>, sqlx::Error> {
    // Inner query: grab the N most-recent rows (DESC).
    // Outer query: flip back to chronological order (ASC) for the LLM.
    let rows = sqlx::query(
        r#"
        SELECT message, is_bot_message
        FROM (
            SELECT message, is_bot_message, created_at
            FROM Conversation
            WHERE user_id = ?1
              AND message_type = 'text'
            ORDER BY created_at DESC
            LIMIT ?2
        )
        ORDER BY created_at ASC
        "#,
    )
    .bind(internal_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let history = rows
        .into_iter()
        .map(|row| ConversationMessage {
            message: row.get::<String, _>("message"),
            // SQLite stores BOOLEAN as INTEGER; 0 = false, anything else = true.
            is_bot_message: row.get::<i64, _>("is_bot_message") != 0,
        })
        .collect();

    Ok(history)
}
