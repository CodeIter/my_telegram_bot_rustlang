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
/// n rows ≈ n/2 user/assistant turns.
const HISTORY_LIMIT: i64 = 100;

// ── NVIDIA / OpenAI-compatible response types

#[derive(Deserialize)]
struct NvidiaChatResponse {
    choices: Vec<NvidiaChoice>,
}

#[derive(Deserialize)]
struct NvidiaChoice {
    message: NvidiaMessage,
}

#[derive(Deserialize)]
struct NvidiaMessage {
    content: String,
}

// ── Handler

pub async fn handle_glm5(
    bot: Bot,
    msg: Message,
    prompt: String,
    enable_thinking: bool,
    pool: SqlitePool,
) -> ResponseResult<()> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        let cmd_name = if enable_thinking {
            "glm5aireasoning"
        } else {
            "glm5ai"
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

    let api_key = match env::var("NVIDIA_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            reply_markdown(
                bot,
                msg,
                "❌ NVIDIA_API_KEY environment variable is not set.\n\
                 Please add it to your .env file and restart the bot."
                    .to_string(),
                &pool,
            )
            .await?;
            return Ok(());
        }
    };

    // ── Build messages[] from conversation history
    let mut messages: Vec<Value> = build_openai_history(&pool, &msg, HISTORY_LIMIT).await;

    // Append the current user prompt as the final turn.
    messages.push(json!({ "role": "user", "content": trimmed }));

    // ── Call NVIDIA API
    let client = reqwest::Client::new();
    let url = "https://integrate.api.nvidia.com/v1/chat/completions";

    let body = json!({
        "model": "z-ai/glm5",
        "messages": messages,
        "temperature": 1,
        "top_p": 1,
        "max_tokens": 16384,
        "seed": 42,
        "stream": false,
        "chat_template_kwargs": {
            "enable_thinking": enable_thinking,
            "clear_thinking": true
        }
    });

    let res = match client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Network error while contacting NVIDIA API: {}", e),
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
            format!("❌ NVIDIA API error (HTTP {}): {}", status, err_text),
            &pool,
        )
        .await?;
        return Ok(());
    }

    let nvidia_resp: NvidiaChatResponse = match res.json().await {
        Ok(parsed) => parsed,
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Failed to parse NVIDIA API response: {}", e),
                &pool,
            )
            .await?;
            return Ok(());
        }
    };

    let response_text = nvidia_resp
        .choices
        .first()
        .map(|c| c.message.content.as_str())
        .unwrap_or("No response text from GLM-5.");

    let model_display = if enable_thinking {
        "GLM-5 AI (with reasoning)"
    } else {
        "GLM-5 AI (without reasoning)"
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
/// `messages` array format used by the OpenAI-compatible NVIDIA API.
///
/// Returns an empty `Vec` if the user cannot be identified or history
/// retrieval fails — the caller still sends the current prompt alone.
async fn build_openai_history(pool: &SqlitePool, msg: &Message, limit: i64) -> Vec<Value> {
    let user = match &msg.from {
        Some(u) => u,
        None => return Vec::new(),
    };

    let internal_id = match upsert_user_and_get_id(pool, user).await {
        Ok(id) => id,
        Err(e) => {
            log::warn!("handle_glm5: failed to get user id for history: {e}");
            return Vec::new();
        }
    };

    let history = match get_conversation_history(pool, internal_id, limit).await {
        Ok(h) => h,
        Err(e) => {
            log::warn!("handle_glm5: failed to fetch conversation history: {e}");
            return Vec::new();
        }
    };

    history
        .into_iter()
        .map(|entry| {
            // OpenAI-compatible API uses "user" / "assistant" roles.
            let role = if entry.is_bot_message {
                "assistant"
            } else {
                "user"
            };
            json!({ "role": role, "content": entry.message })
        })
        .collect()
}
