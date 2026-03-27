use crate::utils::reply_markdown::reply_markdown;
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::env;
use teloxide::{Bot, prelude::*, types::Message};

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

pub async fn handle_gemini(
    bot: Bot,
    msg: Message,
    prompt: String,
    model: &str,
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
            )
            .await?;
            return Ok(());
        }
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );

    let body = json!({
        "contents": [{
            "parts": [{
                "text": trimmed
            }]
        }]
    });

    let res = match client.post(&url).json(&body).send().await {
        Ok(response) => response,
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Network error while contacting Gemini API: {}", e),
            )
            .await?;
            return Ok(());
        }
    };

    if !res.status().is_success() {
        let status = res.status();
        let err_text = match res.text().await {
            Ok(text) => text,
            Err(_) => "Unknown error".to_string(),
        };
        reply_markdown(
            bot,
            msg,
            format!("❌ Gemini API error (HTTP {}): {}", status, err_text),
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
            )
            .await?;
            return Ok(());
        }
    };

    let response_text = gemini_resp
        .candidates
        .first()
        .and_then(|candidate| candidate.content.parts.first())
        .map(|part| part.text.as_str())
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
    )
    .await?;

    Ok(())
}
