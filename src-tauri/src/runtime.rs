use std::{ffi::OsStr, path::Path, process::Command};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub(crate) struct CommandVersion {
    pub ok: Option<String>,
    pub err: Option<String>,
}

pub(crate) fn command_version(command: &str, args: &[&str]) -> CommandVersion {
    match hidden_command(command).args(args).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            CommandVersion {
                ok: Some(stdout),
                err: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            CommandVersion {
                ok: None,
                err: Some(if stderr.is_empty() {
                    format!("{command} exited with status {}", output.status)
                } else {
                    stderr
                }),
            }
        }
        Err(err) => CommandVersion {
            ok: None,
            err: Some(err.to_string()),
        },
    }
}

pub(crate) fn hidden_command<S: AsRef<OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    hide_command_window(&mut command);
    command
}

pub(crate) fn hide_command_window(command: &mut Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(crate) fn pm2_execution_enabled() -> bool {
    cfg!(windows) || std::env::var("EDUPORTAL_CONTROL_RUN_PM2").ok().as_deref() == Some("1")
}

pub(crate) fn pm2_command() -> &'static str {
    if cfg!(windows) {
        "pm2.cmd"
    } else {
        "pm2"
    }
}

pub(crate) fn node_command() -> &'static str {
    if cfg!(windows) {
        "node.exe"
    } else {
        "node"
    }
}

pub(crate) fn npm_command() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(crate) fn join_display_path(base: &str, parts: &[&str]) -> String {
    let separator = if looks_windows_path(base) { "\\" } else { "/" };
    let mut path = base.trim_end_matches(['\\', '/']).to_string();
    for part in parts {
        if !path.is_empty() {
            path.push_str(separator);
        }
        path.push_str(part.trim_matches(['\\', '/']));
    }
    path
}

pub(crate) fn looks_windows_path(path: &str) -> bool {
    path.contains('\\') || path.as_bytes().get(1) == Some(&b':')
}

pub(crate) async fn run_blocking<T, F>(label: &'static str, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("{label} worker failed: {err}"))?
}
