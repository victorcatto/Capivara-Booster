use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, serde::Serialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct ProcessExecutor;

impl ProcessExecutor {
    pub async fn execute_cmd(command: &str) -> ExecutionResult {
        run("cmd.exe", &["/C", command]).await
    }

    pub async fn execute_powershell(script: &str) -> ExecutionResult {
        run(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
        ).await
    }
}

async fn run(program: &str, args: &[&str]) -> ExecutionResult {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

    let stdout = child.stdout.take().expect("stdout not captured");
    let stderr = child.stderr.take().expect("stderr not captured");

    let stdout_task = tokio::spawn(collect(stdout));
    let stderr_task = tokio::spawn(collect(stderr));

    let status = child.wait().await;
    let out = stdout_task.await.unwrap_or_default();
    let err = stderr_task.await.unwrap_or_default();

    ExecutionResult {
        exit_code: status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1),
        stdout: out,
        stderr: err,
    }
}

async fn collect<T: tokio::io::AsyncRead + Unpin>(stream: T) -> String {
    let mut lines = BufReader::new(stream).lines();
    let mut result = String::new();
    while let Ok(Some(line)) = lines.next_line().await {
        result.push_str(&line);
        result.push('\n');
    }
    result
}
