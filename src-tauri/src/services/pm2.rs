use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    domain::{DeploymentState, Pm2CommandResult, Pm2Process, Settings, CADDY_APP_NAME},
    runtime::{display_path, join_display_path, pm2_command, pm2_execution_enabled},
};

#[derive(Debug, Serialize)]
pub(crate) struct Pm2Config {
    pub apps: Vec<Pm2App>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Pm2App {
    pub name: String,
    pub script: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpreter: Option<String>,
    pub cwd: String,
    pub log_file: String,
    pub error_file: String,
    pub log_date_format: String,
    pub merge_logs: bool,
    pub env: BTreeMap<String, String>,
}

pub(crate) fn build_pm2_config(settings: &Settings, deployment_path: &str) -> Pm2Config {
    let portal_log_file = join_display_path(&settings.deploy_root, &["pm2", "logs", "Portal.log"]);
    let web_api_log_file = join_display_path(&settings.deploy_root, &["pm2", "logs", "WebApi.log"]);

    Pm2Config {
        apps: vec![
            Pm2App {
                name: "Portal".to_string(),
                script: join_display_path(deployment_path, &["Portal", "build", "index.js"]),
                args: None,
                interpreter: None,
                cwd: join_display_path(deployment_path, &["Portal"]),
                log_file: portal_log_file.clone(),
                error_file: portal_log_file,
                log_date_format: "YYYY-MM-DD HH:mm:ss".to_string(),
                merge_logs: true,
                env: settings.portal_env.clone(),
            },
            Pm2App {
                name: "WebApi".to_string(),
                script: join_display_path(deployment_path, &["WebApi", "WebApi.exe"]),
                args: None,
                interpreter: None,
                cwd: join_display_path(deployment_path, &["WebApi"]),
                log_file: web_api_log_file.clone(),
                error_file: web_api_log_file,
                log_date_format: "YYYY-MM-DD HH:mm:ss".to_string(),
                merge_logs: true,
                env: settings.web_api_env.clone(),
            },
        ],
    }
}

pub(crate) fn build_caddy_pm2_config(
    settings: &Settings,
    executable_path: &str,
    config_path: &str,
    install_dir: &str,
) -> Pm2Config {
    let log_file = join_display_path(&settings.deploy_root, &["pm2", "logs", "Caddy.log"]);

    Pm2Config {
        apps: vec![Pm2App {
            name: CADDY_APP_NAME.to_string(),
            script: executable_path.to_string(),
            args: Some(vec![
                "run".to_string(),
                "--config".to_string(),
                config_path.to_string(),
                "--adapter".to_string(),
                "caddyfile".to_string(),
            ]),
            interpreter: Some("none".to_string()),
            cwd: install_dir.to_string(),
            log_file: log_file.clone(),
            error_file: log_file,
            log_date_format: "YYYY-MM-DD HH:mm:ss".to_string(),
            merge_logs: true,
            env: BTreeMap::new(),
        }],
    }
}

pub(crate) fn apply_retention(state: &mut DeploymentState, retention: usize) -> Vec<String> {
    let mut sorted = state.deployments.clone();
    sorted.sort_by(|a, b| b.id.cmp(&a.id));

    let mut keep = BTreeSet::new();
    if let Some(active) = &state.active_deployment_id {
        keep.insert(active.clone());
    }
    for deployment in sorted.iter().take(retention) {
        keep.insert(deployment.id.clone());
    }

    let mut removed = Vec::new();
    state.deployments.retain(|deployment| {
        if keep.contains(&deployment.id) {
            true
        } else {
            let path = PathBuf::from(&deployment.deployment_path);
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
            }
            removed.push(deployment.id.clone());
            false
        }
    });
    removed
}

pub(crate) fn run_pm2_config(config_path: &Path) -> Pm2CommandResult {
    let mut command_label = format!(
        "{} startOrReload {} --update-env",
        pm2_command(),
        display_path(config_path)
    );

    if !pm2_execution_enabled() {
        return Pm2CommandResult {
            attempted: false,
            skipped: true,
            success: true,
            command: command_label,
            stdout: String::new(),
            stderr: String::new(),
            message: "PM2 execution is skipped outside Windows. Set EDUPORTAL_CONTROL_RUN_PM2=1 to force it.".to_string(),
        };
    }

    match Command::new(pm2_command())
        .arg("startOrReload")
        .arg(config_path)
        .arg("--update-env")
        .output()
    {
        Ok(output) => {
            let mut stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let success = output.status.success();
            let mut message = if success {
                "PM2 reload completed".to_string()
            } else if stderr.is_empty() {
                format!("PM2 exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            if success {
                command_label.push_str(&format!(" && {} save", pm2_command()));
                match Command::new(pm2_command()).arg("save").output() {
                    Ok(save_output) => {
                        let save_stdout = String::from_utf8_lossy(&save_output.stdout)
                            .trim()
                            .to_string();
                        let save_stderr = String::from_utf8_lossy(&save_output.stderr)
                            .trim()
                            .to_string();
                        stdout = join_command_output(&stdout, &save_stdout);
                        stderr = join_command_output(&stderr, &save_stderr);
                        if save_output.status.success() {
                            message = "PM2 reload completed and process list saved.".to_string();
                        } else if save_stderr.is_empty() {
                            message = format!(
                                "PM2 reload completed, but pm2 save exited with status {}.",
                                save_output.status
                            );
                        } else {
                            message =
                                format!("PM2 reload completed, but pm2 save failed: {save_stderr}");
                        }
                    }
                    Err(err) => {
                        stderr = join_command_output(&stderr, &err.to_string());
                        message = format!("PM2 reload completed, but pm2 save failed: {err}");
                    }
                }
            }

            Pm2CommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                stdout,
                stderr,
                message,
            }
        }
        Err(err) => Pm2CommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to execute PM2: {err}"),
        },
    }
}

pub(crate) fn run_pm2_config_with_recovery(
    config_path: &Path,
    deployment_path: &str,
    list_pm2_processes_blocking: fn() -> Result<Vec<Pm2Process>, String>,
) -> Pm2CommandResult {
    let result = run_pm2_config(config_path);
    if !result.attempted || result.skipped || !result.success {
        return result;
    }

    match verify_pm2_deployment_path(deployment_path, list_pm2_processes_blocking) {
        Ok(()) => result,
        Err(first_mismatch) => {
            let delete_result = run_pm2_delete_managed_apps();
            let mut combined = combine_pm2_results(
                result,
                delete_result.clone(),
                true,
                format!(
                    "PM2 pointed to an unexpected deployment after reload ({first_mismatch}); recreating managed apps."
                ),
            );

            if delete_result.attempted && !delete_result.success {
                combined.success = false;
                combined.message = format!(
                    "PM2 reload completed, but active process path was stale and PM2 cleanup failed: {}",
                    delete_result.message
                );
                return combined;
            }

            let retry = run_pm2_config(config_path);
            combined = combine_pm2_results(combined, retry.clone(), retry.success, retry.message);
            if !retry.success || retry.skipped || !retry.attempted {
                return combined;
            }

            match verify_pm2_deployment_path(deployment_path, list_pm2_processes_blocking) {
                Ok(()) => Pm2CommandResult {
                    success: true,
                    message: "PM2 apps were recreated with the active deployment config.".to_string(),
                    ..combined
                },
                Err(second_mismatch) => Pm2CommandResult {
                    success: false,
                    message: format!(
                        "PM2 apps still point to an unexpected deployment after recreate: {second_mismatch}"
                    ),
                    ..combined
                },
            }
        }
    }
}

fn verify_pm2_deployment_path(
    deployment_path: &str,
    list_pm2_processes_blocking: fn() -> Result<Vec<Pm2Process>, String>,
) -> Result<(), String> {
    if !pm2_execution_enabled() {
        return Ok(());
    }

    let processes = list_pm2_processes_blocking()?;
    let expected = [
        (
            "Portal",
            join_display_path(deployment_path, &["Portal"]),
            join_display_path(deployment_path, &["Portal", "build", "index.js"]),
        ),
        (
            "WebApi",
            join_display_path(deployment_path, &["WebApi"]),
            join_display_path(deployment_path, &["WebApi", "WebApi.exe"]),
        ),
    ];

    for (name, expected_cwd, expected_script) in expected {
        let process = processes
            .iter()
            .find(|process| process.name == name)
            .ok_or_else(|| format!("{name} is not registered in PM2"))?;
        let cwd = process
            .cwd
            .as_deref()
            .ok_or_else(|| format!("{name} has no PM2 cwd"))?;
        if !pm2_paths_match(cwd, &expected_cwd) {
            return Err(format!("{name} cwd is {}, expected {}", cwd, expected_cwd));
        }

        let script_path = process
            .script_path
            .as_deref()
            .ok_or_else(|| format!("{name} has no PM2 script path"))?;
        if !pm2_paths_match(script_path, &expected_script) {
            return Err(format!(
                "{name} script is {}, expected {}",
                script_path, expected_script
            ));
        }
    }

    Ok(())
}

pub(crate) fn pm2_paths_match(actual: &str, expected: &str) -> bool {
    normalize_pm2_path(actual) == normalize_pm2_path(expected)
}

fn normalize_pm2_path(path: &str) -> String {
    path.trim()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn run_pm2_delete_managed_apps() -> Pm2CommandResult {
    let command_label = format!("{} delete Portal WebApi", pm2_command());

    if !pm2_execution_enabled() {
        return Pm2CommandResult {
            attempted: false,
            skipped: true,
            success: true,
            command: command_label,
            stdout: String::new(),
            stderr: String::new(),
            message: "PM2 execution is skipped outside Windows. Set EDUPORTAL_CONTROL_RUN_PM2=1 to force it."
                .to_string(),
        };
    }

    match Command::new(pm2_command())
        .arg("delete")
        .arg("Portal")
        .arg("WebApi")
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let output_text = format!("{stdout}\n{stderr}").to_lowercase();
            let success = output.status.success() || output_text.contains("not found");
            let message = if success {
                "PM2 managed apps were cleared before restart.".to_string()
            } else if stderr.is_empty() {
                format!("PM2 delete exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Pm2CommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                stdout,
                stderr,
                message,
            }
        }
        Err(err) => Pm2CommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to delete PM2 managed apps: {err}"),
        },
    }
}

fn combine_pm2_results(
    first: Pm2CommandResult,
    second: Pm2CommandResult,
    success: bool,
    message: String,
) -> Pm2CommandResult {
    Pm2CommandResult {
        attempted: first.attempted || second.attempted,
        skipped: first.skipped && second.skipped,
        success,
        command: join_command_output(&first.command, &second.command),
        stdout: join_command_output(&first.stdout, &second.stdout),
        stderr: join_command_output(&first.stderr, &second.stderr),
        message,
    }
}

pub(crate) fn run_pm2_app_action(action: &str, app_name: &str) -> Pm2CommandResult {
    let command_label = format!("{} {} {}", pm2_command(), action, app_name);

    if !pm2_execution_enabled() {
        return Pm2CommandResult {
            attempted: false,
            skipped: true,
            success: true,
            command: command_label,
            stdout: String::new(),
            stderr: String::new(),
            message: "PM2 execution is skipped outside Windows. Set EDUPORTAL_CONTROL_RUN_PM2=1 to force it."
                .to_string(),
        };
    }

    match Command::new(pm2_command())
        .arg(action)
        .arg(app_name)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let success = output.status.success();
            let message = if success {
                format!("PM2 {action} completed for {app_name}.")
            } else if stderr.is_empty() {
                format!("PM2 {action} exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Pm2CommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                stdout,
                stderr,
                message,
            }
        }
        Err(err) => Pm2CommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to execute PM2 {action}: {err}"),
        },
    }
}

fn join_command_output(existing: &str, next: &str) -> String {
    match (existing.is_empty(), next.is_empty()) {
        (true, true) => String::new(),
        (true, false) => next.to_string(),
        (false, true) => existing.to_string(),
        (false, false) => format!("{existing}\n{next}"),
    }
}

pub(crate) fn validate_pm2_app_name(app_name: &str) -> Result<&'static str, String> {
    match app_name {
        "Portal" => Ok("Portal"),
        "WebApi" => Ok("WebApi"),
        CADDY_APP_NAME => Ok(CADDY_APP_NAME),
        _ => Err("Unknown PM2 app. Expected Portal, WebApi, or Caddy.".to_string()),
    }
}

pub(crate) fn validate_pm2_action(action: &str) -> Result<&'static str, String> {
    match action {
        "start" => Ok("start"),
        "stop" => Ok("stop"),
        "restart" => Ok("restart"),
        _ => Err("Unknown PM2 action. Expected start, stop, or restart.".to_string()),
    }
}

pub(crate) fn parse_pm2_processes(json: &str) -> Result<Vec<Pm2Process>, String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|err| format!("Failed to parse PM2 jlist JSON: {err}"))?;
    let items = value
        .as_array()
        .ok_or_else(|| "PM2 jlist did not return an array".to_string())?;

    let mut processes = Vec::new();
    for item in items {
        let Some(name) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let pm2_env = item.get("pm2_env").unwrap_or(&serde_json::Value::Null);
        let monit = item.get("monit").unwrap_or(&serde_json::Value::Null);

        processes.push(Pm2Process {
            name: name.to_string(),
            pm_id: item.get("pm_id").and_then(|value| value.as_i64()),
            status: pm2_env
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string(),
            pid: item.get("pid").and_then(|value| value.as_i64()),
            restart_time: pm2_env.get("restart_time").and_then(|value| value.as_i64()),
            unstable_restarts: pm2_env
                .get("unstable_restarts")
                .and_then(|value| value.as_i64()),
            cpu: monit.get("cpu").and_then(|value| value.as_f64()),
            memory: monit.get("memory").and_then(|value| value.as_u64()),
            uptime: pm2_env.get("pm_uptime").and_then(|value| value.as_i64()),
            script_path: pm2_env
                .get("pm_exec_path")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned),
            cwd: pm2_env
                .get("pm_cwd")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned),
        });
    }

    Ok(processes)
}
