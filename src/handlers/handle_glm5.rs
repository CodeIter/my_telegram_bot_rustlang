use crate::utils::reply_markdown::reply_markdown;
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::env;
use teloxide::{Bot, prelude::*, types::Message};

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

pub async fn handle_glm5(
    bot: Bot,
    msg: Message,
    prompt: String,
    enable_thinking: bool,
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
                "❌ NVIDIA_API_KEY environment variable is not set.\nPlease add it to your .env file and restart the bot.".to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let client = reqwest::Client::new();
    let url = "https://integrate.api.nvidia.com/v1/chat/completions";

    let body = json!({
        "model": "z-ai/glm5",
        "messages": [
            {
                "role": "user",
                "content": trimmed
            }
        ],
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
            format!("❌ NVIDIA API error (HTTP {}): {}", status, err_text),
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
            )
            .await?;
            return Ok(());
        }
    };

    let response_text = nvidia_resp
        .choices
        .first()
        .map(|choice| choice.message.content.as_str())
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
    )
    .await?;

    Ok(())
}
