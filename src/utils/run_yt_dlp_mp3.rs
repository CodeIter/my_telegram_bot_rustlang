use tokio::process::Command as TokioProcessCommand;

pub async fn run_yt_dlp_mp3(
    url: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let child = TokioProcessCommand::new("yt-dlp")
        .arg("--quiet")
        .arg("--no-warnings")
        .arg("--no-playlist")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("-o")
        .arg(output)
        .arg(url)
        .spawn()?;

    let output = child.wait_with_output().await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("yt-dlp exit code: {:?}", output.status.code()).into())
    }
}
