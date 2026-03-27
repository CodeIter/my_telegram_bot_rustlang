use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Say hello!")]
    Start,
    #[command(description = "Echo any text (but we also echo without command)")]
    Echo(String),

    #[command(description = "/urldecode <encoded> → decode URL")]
    UrlDecode(String),

    #[command(description = "/textbase64encode <text> → encode to base64")]
    TextBase64Encode(String),

    #[command(description = "/textbase64decode <text> → decode base64")]
    TextBase64Decode(String),

    #[command(
        description = "/rng <min> <max> → random number (min > 0)",
        parse_with = "split"
    )]
    Rng(u32, u32),

    #[command(description = "/password <length> → generate password (>1)")]
    Password(u32),

    #[command(description = "/bc <expression> → calculate with bc (e.g. /bc 2+2*3)")]
    Bc(String),

    #[command(description = "/ytdl <url> → download & send video with yt-dlp")]
    Ytdl(String),

    #[command(description = "/ytdlmp3 <url> → download & send as MP3 with yt-dlp")]
    YtdlMp3(String),

    #[command(description = "/textqr <text> → generate QR code image")]
    TextQr(String),

    #[command(description = "/gemini3 <prompt> → ask Gemini 3 Flash Preview AI")]
    Gemini3(String),

    #[command(description = "/gemini2 <prompt> → ask Gemini 2.5 Flash AI")]
    Gemini2(String),

    #[command(description = "/glm5ai <prompt> → ask GLM-5 AI without reasoning")]
    Glm5Ai(String),

    #[command(description = "/glm5aireasoning <prompt> → ask GLM-5 AI with reasoning")]
    Glm5AiReasoning(String),
}
