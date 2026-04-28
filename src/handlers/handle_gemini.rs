use crate::utils::get_conversation_history::get_conversation_history;
use crate::utils::reply_markdown::reply_markdown;
use crate::utils::upsert_user::upsert_user_and_get_id;
use reqwest;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::env;
use teloxide::{Bot, prelude::*, types::Message};

/// Maximum number of past messages sent as context to the model.
/// n rows ≈ n/2 user/model turns.
const HISTORY_LIMIT: i64 = 100;

// ── Gemini response types

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

// ── Handler

pub async fn handle_gemini(
    bot: Bot,
    msg: Message,
    prompt: String,
    model: &str,
    pool: SqlitePool,
) -> ResponseResult<()> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        let cmd_name = if model == "gemini-3-flash-preview" {
            "gemini3"
        } else {
            "gemini2"
        };
        reply_markdown(
            bot,
            msg,
            format!("❌ Usage: /{cmd_name} <your prompt here> (prompt is required)"),
            &pool,
        )
        .await?;
        return Ok(());
    }

    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            reply_markdown(
                bot,
                msg,
                "❌ GEMINI_API_KEY environment variable is not set.\nPlease add it to your .env file and restart the bot.".to_string(),
                &pool,
            )
            .await?;
            return Ok(());
        }
    };

    // ── Build contents[] from conversation history
    let mut contents: Vec<Value> = build_gemini_history(&pool, &msg, HISTORY_LIMIT).await;

    // Append the current user prompt as the final turn.
    contents.push(json!({
        "role": "user",
        "parts": [{ "text": trimmed }]
    }));

    // ── Call Gemini API
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );

    let res = match client
        .post(&url)
        .json(&json!({ "contents": contents }))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Network error while contacting Gemini API: {}", e),
                &pool,
            )
            .await?;
            return Ok(());
        }
    };

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        reply_markdown(
            bot,
            msg,
            format!("❌ Gemini API error (HTTP {}): {}", status, err_text),
            &pool,
        )
        .await?;
        return Ok(());
    }

    let gemini_resp: GeminiResponse = match res.json().await {
        Ok(parsed) => parsed,
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Failed to parse Gemini API response: {}", e),
                &pool,
            )
            .await?;
            return Ok(());
        }
    };

    let response_text = gemini_resp
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.as_str())
        .unwrap_or("No response text from Gemini.");

    let model_display = if model == "gemini-3-flash-preview" {
        "Gemini 3 Flash Preview"
    } else {
        "Gemini 2.5 Flash"
    };

    reply_markdown(
        bot,
        msg,
        format!("🤖 {}:\n{}", model_display, response_text),
        &pool,
    )
    .await?;

    Ok(())
}

// ── Private helpers

/// Fetches the user's stored conversation and converts it to the
/// `contents` array format expected by the Gemini API.
///
/// Returns an empty `Vec` if the user cannot be identified or history
/// retrieval fails — the caller still sends the current prompt alone.
async fn build_gemini_history(pool: &SqlitePool, msg: &Message, limit: i64) -> Vec<Value> {
    let user = match &msg.from {
        Some(u) => u,
        None => return Vec::new(),
    };

    let internal_id = match upsert_user_and_get_id(pool, user).await {
        Ok(id) => id,
        Err(e) => {
            log::warn!("handle_gemini: failed to get user id for history: {e}");
            return Vec::new();
        }
    };

    let history = match get_conversation_history(pool, internal_id, limit).await {
        Ok(h) => h,
        Err(e) => {
            log::warn!("handle_gemini: failed to fetch conversation history: {e}");
            return Vec::new();
        }
    };

    history
        .into_iter()
        .map(|entry| {
            // Gemini uses "user" / "model" roles.
            let role = if entry.is_bot_message {
                "model"
            } else {
                "user"
            };
            json!({
                "role": role,
                "parts": [{ "text": entry.message }]
            })
        })
        .collect()
}
