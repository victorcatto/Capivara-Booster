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
    pub async fn execute_cmd(&self, command: &str) -> ExecutionResult {
        run("cmd.exe", &["/C", command]).await
    }

    pub async fn execute_powershell(&self, script: &str) -> ExecutionResult {
        run(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
        ).await
    }
}

async fn run(program: &str, args: &[&str]) -> ExecutionResult {
    use std::os::windows::process::CommandExt;
    let child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // CREATE_NO_WINDOW: o app roda os scripts sem abrir console visível.
        .creation_flags(0x0800_0000)
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return ExecutionResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("falha ao iniciar {program}: {e}"),
            };
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_task = tokio::spawn(async move {
        match stdout {
            Some(s) => collect(s).await,
            None => String::new(),
        }
    });
    let stderr_task = tokio::spawn(async move {
        match stderr {
            Some(s) => collect(s).await,
            None => String::new(),
        }
    });

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
