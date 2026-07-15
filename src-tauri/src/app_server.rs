use crate::usage::{
    build_ready_snapshot, Account, AccountResponse, RateLimitsResponse, SnapshotMetadata,
    TokenUsageResponse, UsageSnapshot,
};
use chrono::{Local, SecondsFormat, Utc};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::env;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Error)]
pub(crate) enum AppServerError {
    #[error("未找到 Codex CLI。请先安装或更新 Codex，并确认 codex 命令可用。")]
    CodexNotFound,
    #[error("无法启动 Codex app-server：{0}")]
    Spawn(String),
    #[error("Codex app-server 通信失败：{0}")]
    Io(String),
    #[error("Codex app-server 返回了无效数据：{0}")]
    Protocol(String),
    #[error("等待 Codex app-server 的 {0} 响应超时。")]
    Timeout(String),
    #[error("Codex app-server 拒绝了 {method}：{message}")]
    Rpc { method: String, message: String },
}

struct RpcSession {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    receiver: Receiver<Result<Value, String>>,
    next_id: u64,
}

impl RpcSession {
    fn start() -> Result<Self, AppServerError> {
        let executable = discover_codex().ok_or(AppServerError::CodexNotFound)?;
        let mut command = codex_command(&executable);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }

        let mut child = command
            .spawn()
            .map_err(|error| AppServerError::Spawn(error.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppServerError::Spawn("没有获得标准输入".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppServerError::Spawn("没有获得标准输出".to_string()))?;
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let message = match line {
                    Ok(line) if line.trim().is_empty() => continue,
                    Ok(line) => serde_json::from_str::<Value>(&line)
                        .map_err(|error| format!("{error}; JSONL={}", truncate(&line, 240))),
                    Err(error) => Err(error.to_string()),
                };
                if sender.send(message).is_err() {
                    break;
                }
            }
        });

        let mut session = Self {
            child,
            stdin: BufWriter::new(stdin),
            receiver,
            next_id: 1,
        };
        let _: Value = session.request(
            "initialize",
            Some(json!({
              "clientInfo": {
                "name": "hexu_codex_usage",
                "title": "禾序 · Codex 用量助手",
                "version": env!("CARGO_PKG_VERSION")
              },
              "capabilities": null
            })),
        )?;
        session.notify("initialized", Some(json!({})))?;
        Ok(session)
    }

    fn request<T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<T, AppServerError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut request = Map::new();
        request.insert("method".to_string(), Value::String(method.to_string()));
        request.insert("id".to_string(), Value::from(id));
        if let Some(params) = params {
            request.insert("params".to_string(), params);
        }
        self.write(Value::Object(request))?;

        let deadline = Instant::now() + REQUEST_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(AppServerError::Timeout(method.to_string()));
            }
            let message = match self.receiver.recv_timeout(remaining) {
                Ok(message) => message.map_err(AppServerError::Protocol)?,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(AppServerError::Timeout(method.to_string()));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    let status = self
                        .child
                        .try_wait()
                        .ok()
                        .flatten()
                        .map(|status| status.to_string())
                        .unwrap_or_else(|| "未知状态".to_string());
                    return Err(AppServerError::Protocol(format!(
                        "app-server 在响应 {method} 前退出（{status}）"
                    )));
                }
            };
            if message.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                let rpc_message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("未知错误");
                return Err(AppServerError::Rpc {
                    method: method.to_string(),
                    message: truncate(rpc_message, 320),
                });
            }
            let result = message
                .get("result")
                .cloned()
                .ok_or_else(|| AppServerError::Protocol(format!("{method} 缺少 result")))?;
            return serde_json::from_value(result)
                .map_err(|error| AppServerError::Protocol(format!("{method}: {error}")));
        }
    }

    fn notify(&mut self, method: &str, params: Option<Value>) -> Result<(), AppServerError> {
        let mut notification = Map::new();
        notification.insert("method".to_string(), Value::String(method.to_string()));
        if let Some(params) = params {
            notification.insert("params".to_string(), params);
        }
        self.write(Value::Object(notification))
    }

    fn write(&mut self, value: Value) -> Result<(), AppServerError> {
        serde_json::to_writer(&mut self.stdin, &value)
            .map_err(|error| AppServerError::Io(error.to_string()))?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| AppServerError::Io(error.to_string()))
    }
}

impl Drop for RpcSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(crate) fn fetch_usage_snapshot() -> Result<UsageSnapshot, AppServerError> {
    let mut session = RpcSession::start()?;
    let initialize_user_agent =
        session.request::<Value>("account/read", Some(json!({ "refreshToken": false })))?;
    let account_response: AccountResponse = serde_json::from_value(initialize_user_agent)
        .map_err(|error| AppServerError::Protocol(format!("account/read: {error}")))?;
    let source = "Codex app-server".to_string();
    let updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let codex_version = codex_version();

    let Some(account) = account_response.account.as_ref() else {
        return Ok(UsageSnapshot::signed_out(
            account_response.requires_openai_auth,
            source,
            updated_at,
            codex_version,
        ));
    };

    if !matches!(account, Account::ChatGpt { .. }) {
        return Ok(UsageSnapshot::unsupported(
            account,
            source,
            updated_at,
            codex_version,
        ));
    }

    let rate_limits: RateLimitsResponse = session.request("account/rateLimits/read", None)?;
    let (token_usage, usage_note) =
        match session.request::<TokenUsageResponse>("account/usage/read", None) {
            Ok(usage) => (Some(usage), None),
            Err(error) => (
                None,
                Some(format!("额度已读取；Token 用量暂不可用：{error}")),
            ),
        };

    Ok(build_ready_snapshot(
        account,
        rate_limits,
        token_usage,
        usage_note,
        SnapshotMetadata {
            today: Local::now().date_naive(),
            source,
            updated_at,
            codex_version,
        },
    ))
}

fn discover_codex() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os("HEXU_CODEX_BIN") {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(path) = find_on_path() {
        return native_codex_from_npm_launcher(&path).or(Some(path));
    }

    #[cfg(windows)]
    if let Some(app_data) = env::var_os("APPDATA") {
        let npm = PathBuf::from(app_data).join("npm");
        for name in ["codex.exe", "codex.cmd", "codex.bat"] {
            let candidate = npm.join(name);
            if candidate.is_file() {
                return native_codex_from_npm_launcher(&candidate).or(Some(candidate));
            }
        }
    }

    None
}

#[cfg(windows)]
fn native_codex_from_npm_launcher(launcher: &Path) -> Option<PathBuf> {
    let extension = launcher.extension().and_then(OsStr::to_str)?;
    if !matches!(
        extension.to_ascii_lowercase().as_str(),
        "cmd" | "bat" | "ps1"
    ) {
        return None;
    }
    let npm_root = launcher.parent()?;
    let (package, target) = match env::consts::ARCH {
        "x86_64" => ("codex-win32-x64", "x86_64-pc-windows-msvc"),
        "aarch64" => ("codex-win32-arm64", "aarch64-pc-windows-msvc"),
        _ => return None,
    };
    let candidate = npm_root
        .join("node_modules")
        .join("@openai")
        .join("codex")
        .join("node_modules")
        .join("@openai")
        .join(package)
        .join("vendor")
        .join(target)
        .join("bin")
        .join("codex.exe");
    candidate.is_file().then_some(candidate)
}

#[cfg(not(windows))]
fn native_codex_from_npm_launcher(_launcher: &Path) -> Option<PathBuf> {
    None
}

fn find_on_path() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    #[cfg(windows)]
    let names = ["codex.exe", "codex.cmd", "codex.bat"];
    #[cfg(not(windows))]
    let names = ["codex"];

    for directory in env::split_paths(&path) {
        for name in names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn codex_command(executable: &Path) -> Command {
    #[cfg(windows)]
    if executable
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "cmd" | "bat"))
    {
        let mut command = Command::new("cmd.exe");
        let escaped = executable.to_string_lossy().replace('"', "\"\"");
        command
            .arg("/d")
            .arg("/s")
            .arg("/c")
            .arg(format!("call \"{escaped}\" app-server --listen stdio://"));
        return command;
    }

    let mut command = Command::new(executable);
    command.args(["app-server", "--listen", "stdio://"]);
    command
}

fn codex_version() -> Option<String> {
    let executable = discover_codex()?;
    let mut command = if executable
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "cmd" | "bat"))
    {
        #[cfg(windows)]
        {
            let mut command = Command::new("cmd.exe");
            let escaped = executable.to_string_lossy().replace('"', "\"\"");
            command
                .arg("/d")
                .arg("/s")
                .arg("/c")
                .arg(format!("call \"{escaped}\" --version"));
            command
        }
        #[cfg(not(windows))]
        {
            Command::new(executable)
        }
    } else {
        let mut command = Command::new(executable);
        command.arg("--version");
        command
    };
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_string())
        .filter(|version| !version.is_empty())
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a local Codex installation and an existing login"]
    fn reads_a_privacy_safe_live_snapshot() {
        let snapshot = fetch_usage_snapshot().expect("live Codex snapshot should be readable");
        let serialized = serde_json::to_string_pretty(&snapshot).unwrap();
        assert!(
            !serialized.contains('@'),
            "the snapshot must not expose email"
        );
        println!("{serialized}");
    }
}
