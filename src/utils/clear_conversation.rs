use sqlx::SqlitePool;

pub async fn clear_user_conversation(
    pool: &SqlitePool,
    internal_user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM Conversation WHERE user_id = ?1")
        .bind(internal_user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
