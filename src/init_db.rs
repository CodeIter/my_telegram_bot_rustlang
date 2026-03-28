use rust_embed::RustEmbed;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;

#[derive(RustEmbed)]
#[folder = "sql/"]
#[include = "*.sql"]
struct Asset;

pub async fn init_db() -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "bot.db".to_string());

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    let mut sql_files: Vec<_> = Asset::iter().collect();
    sql_files.sort_by_key(|f| f.as_ref().to_string());

    for file_path in sql_files {
        if let Some(file) = Asset::get(&file_path) {
            let sql = std::str::from_utf8(&file.data)
                .map_err(|e| format!("Invalid UTF-8 in {}: {}", file_path, e))?;
            if !sql.trim().is_empty() {
                sqlx::query(sql).execute(&pool).await?;
                log::info!("✅ Executed embedded SQL: {}", file_path);
            }
        }
    }

    log::info!("✅ SQLite database initialized ({})", db_path);
    Ok(pool)
}
