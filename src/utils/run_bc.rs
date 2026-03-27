use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioProcessCommand;

pub async fn run_bc(expr: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = TokioProcessCommand::new("bc")
        .arg("-l")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(expr.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
    }

    let output = child.wait_with_output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() && stderr.is_empty() {
        Ok(stdout)
    } else {
        Err(format!(
            "bc error (exit {:?}): {}\nstdout was: {}",
            output.status.code(),
            if stderr.is_empty() {
                "(no stderr)".to_string()
            } else {
                stderr
            },
            stdout.trim()
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bc_calculator_basic() {
        let res = run_bc("2 + 2 * 3").await.unwrap();
        assert_eq!(res.trim(), "8");
    }

    #[tokio::test]
    async fn bc_calculator_with_sqrt() {
        let res = run_bc("scale=0; sqrt(16)").await.unwrap();
        assert_eq!(res.trim(), "4");
    }

    #[tokio::test]
    async fn bc_error_handling() {
        let res = run_bc("syntax error!").await;
        assert!(
            res.is_err(),
            "bc should return Err on invalid input (syntax error reported via stderr)"
        );
    }
}
