use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

use crate::{
    domain::{CaddyCommandResult, CaddyStatus, Settings},
    runtime::{command_version, display_path, hidden_command, looks_windows_path},
    services::pm2,
    storage::write_json,
};

pub(crate) fn caddy_status(settings: &Settings) -> CaddyStatus {
    let install_dir = resolve_caddy_install_dir(settings);
    let executable_path = install_dir.join(caddy_executable_name());
    let config_path = resolve_caddy_config_path(settings);
    let installed = executable_path.is_file();
    let (version, error) = if installed {
        let executable = display_path(&executable_path);
        let version = command_version(&executable, &["version"]);
        (version.ok, version.err)
    } else {
        (None, Some("caddy.exe was not found.".to_string()))
    };

    CaddyStatus {
        installed,
        version,
        error,
        executable_path: display_path(&executable_path),
        install_dir: display_path(&install_dir),
        config_path: display_path(&config_path),
        config_exists: config_path.is_file(),
    }
}

pub(crate) fn install_caddy_zip(
    settings: &Settings,
    zip_path: &Path,
) -> Result<CaddyCommandResult, String> {
    let install_dir = resolve_caddy_install_dir(settings);
    let executable_path = install_dir.join(caddy_executable_name());
    let config_path = resolve_caddy_config_path(settings);
    let command = format!(
        "extract {} -> {}",
        display_path(zip_path),
        display_path(&executable_path)
    );

    fs::create_dir_all(&install_dir).map_err(|err| {
        format!(
            "Failed to create Caddy install directory {}: {err}",
            install_dir.display()
        )
    })?;

    let file = File::open(zip_path)
        .map_err(|err| format!("Failed to open Caddy zip {}: {err}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("Failed to read Caddy zip {}: {err}", zip_path.display()))?;

    let mut copied_entry: Option<String> = None;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("Failed to read Caddy zip entry #{index}: {err}"))?;
        if entry.is_dir() {
            continue;
        }

        let normalized = entry.name().replace('\\', "/");
        let file_name = normalized.rsplit('/').next().unwrap_or_default();
        if !file_name.eq_ignore_ascii_case(caddy_executable_name()) {
            continue;
        }

        let mut out_file = File::create(&executable_path).map_err(|err| {
            format!(
                "Failed to create Caddy executable {}: {err}",
                executable_path.display()
            )
        })?;
        std::io::copy(&mut entry, &mut out_file).map_err(|err| {
            format!(
                "Failed to extract Caddy executable to {}: {err}",
                executable_path.display()
            )
        })?;
        out_file.flush().map_err(|err| {
            format!(
                "Failed to flush Caddy executable {}: {err}",
                executable_path.display()
            )
        })?;
        copied_entry = Some(normalized);
        break;
    }

    if copied_entry.is_some() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create Caddy config directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        if !config_path.exists() {
            fs::write(&config_path, &settings.caddy.config).map_err(|err| {
                format!(
                    "Failed to write default Caddyfile {}: {err}",
                    config_path.display()
                )
            })?;
        }
    }

    let status = caddy_status(settings);
    let success = copied_entry.is_some() && status.installed;
    let message = match copied_entry {
        Some(entry) if success => format!("Caddy extracted from {entry}."),
        Some(entry) => {
            format!("Caddy entry {entry} was copied, but caddy.exe was not found after extraction.")
        }
        None => "Caddy zip is missing caddy.exe.".to_string(),
    };

    Ok(CaddyCommandResult {
        attempted: true,
        skipped: false,
        success,
        command,
        path: Some(display_path(&executable_path)),
        stdout: String::new(),
        stderr: String::new(),
        message,
        status,
        pm2: None,
    })
}

pub(crate) fn apply_caddy_config(settings: &Settings) -> Result<CaddyCommandResult, String> {
    apply_caddy_config_content(
        settings,
        &settings.caddy.config,
        "Caddyfile written and validated. PM2 Caddy is disabled.",
        "Caddyfile validated. PM2 reload skipped outside Windows.",
        "Caddyfile validated and Caddy reloaded through PM2.",
        "Caddyfile validated, but PM2 reload failed",
    )
}

pub(crate) fn apply_caddy_publish_test_config(
    settings: &Settings,
) -> Result<CaddyCommandResult, String> {
    let config = publish_test_caddy_config();
    apply_caddy_config_content(
        settings,
        &config,
        "Publish test Caddyfile written and validated. PM2 Caddy is disabled.",
        "Publish test Caddyfile validated. PM2 reload skipped outside Windows.",
        "Publish test page is active through PM2.",
        "Publish test Caddyfile validated, but PM2 reload failed",
    )
}

fn apply_caddy_config_content(
    settings: &Settings,
    config: &str,
    pm2_disabled_message: &str,
    pm2_skipped_message: &str,
    pm2_success_message: &str,
    pm2_failure_prefix: &str,
) -> Result<CaddyCommandResult, String> {
    let install_dir = resolve_caddy_install_dir(settings);
    let executable_path = install_dir.join(caddy_executable_name());
    let config_path = resolve_caddy_config_path(settings);
    let validate_args = vec![
        "validate".to_string(),
        "--config".to_string(),
        display_path(&config_path),
        "--adapter".to_string(),
        "caddyfile".to_string(),
    ];
    let validate_command = command_label(&executable_path, &validate_args);

    if !executable_path.is_file() {
        return Ok(CaddyCommandResult {
            attempted: false,
            skipped: false,
            success: false,
            command: validate_command,
            path: Some(display_path(&config_path)),
            stdout: String::new(),
            stderr: String::new(),
            message: "Install Caddy before applying the Caddyfile.".to_string(),
            status: caddy_status(settings),
            pm2: None,
        });
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Failed to create Caddy config directory {}: {err}",
                parent.display()
            )
        })?;
    }
    fs::write(&config_path, config)
        .map_err(|err| format!("Failed to write Caddyfile {}: {err}", config_path.display()))?;

    let format_args = vec![
        "fmt".to_string(),
        "--overwrite".to_string(),
        display_path(&config_path),
    ];
    let formatting = run_command(&executable_path, &format_args);
    if !formatting.success {
        return Ok(CaddyCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: formatting.command,
            path: Some(display_path(&config_path)),
            stdout: formatting.stdout,
            stderr: formatting.stderr.clone(),
            message: if formatting.stderr.is_empty() {
                "Caddyfile formatting failed.".to_string()
            } else {
                formatting.stderr
            },
            status: caddy_status(settings),
            pm2: None,
        });
    }

    let validation = run_command(&executable_path, &validate_args);
    let caddy_command = join_command_output(&formatting.command, &validation.command);
    let caddy_stdout = join_command_output(&formatting.stdout, &validation.stdout);
    let caddy_stderr = join_command_output(&formatting.stderr, &validation.stderr);
    if !validation.success {
        return Ok(CaddyCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: caddy_command,
            path: Some(display_path(&config_path)),
            stdout: caddy_stdout,
            stderr: caddy_stderr,
            message: if validation.stderr.is_empty() {
                "Caddyfile validation failed.".to_string()
            } else {
                validation.stderr.clone()
            },
            status: caddy_status(settings),
            pm2: None,
        });
    }

    if !settings.caddy.enabled {
        return Ok(CaddyCommandResult {
            attempted: true,
            skipped: true,
            success: true,
            command: caddy_command,
            path: Some(display_path(&config_path)),
            stdout: caddy_stdout,
            stderr: caddy_stderr,
            message: pm2_disabled_message.to_string(),
            status: caddy_status(settings),
            pm2: None,
        });
    }

    let logs_dir = PathBuf::from(&settings.deploy_root)
        .join("pm2")
        .join("logs");
    fs::create_dir_all(&logs_dir).map_err(|err| {
        format!(
            "Failed to create PM2 log directory {}: {err}",
            logs_dir.display()
        )
    })?;

    let pm2_config_path = install_dir.join("caddy.pm2.json");
    let pm2_config = pm2::build_caddy_pm2_config(
        settings,
        &display_path(&executable_path),
        &display_path(&config_path),
        &display_path(&install_dir),
    );
    write_json(&pm2_config_path, &pm2_config)?;

    let pm2_result = pm2::run_pm2_config(&pm2_config_path);
    let success = pm2_result.success;
    let message = if success {
        if pm2_result.skipped {
            pm2_skipped_message.to_string()
        } else {
            pm2_success_message.to_string()
        }
    } else {
        format!("{pm2_failure_prefix}: {}", pm2_result.message)
    };

    Ok(CaddyCommandResult {
        attempted: true,
        skipped: pm2_result.skipped,
        success,
        command: join_command_output(&caddy_command, &pm2_result.command),
        path: Some(display_path(&config_path)),
        stdout: join_command_output(&caddy_stdout, &pm2_result.stdout),
        stderr: join_command_output(&caddy_stderr, &pm2_result.stderr),
        message,
        status: caddy_status(settings),
        pm2: Some(pm2_result),
    })
}

fn publish_test_caddy_config() -> String {
    r#":80 {
    encode gzip
    header Content-Type "text/plain; charset=utf-8"
    respond "EduClassControl publish test: this VPS is reachable on HTTP port 80." 200
}
"#
    .to_string()
}

pub(crate) fn resolve_caddy_install_dir(settings: &Settings) -> PathBuf {
    resolve_deploy_path(&settings.deploy_root, &settings.caddy.install_dir)
}

pub(crate) fn resolve_caddy_config_path(settings: &Settings) -> PathBuf {
    resolve_deploy_path(&settings.deploy_root, &settings.caddy.config_path)
}

fn resolve_deploy_path(deploy_root: &str, configured: &str) -> PathBuf {
    let path = PathBuf::from(configured);
    if path.is_absolute() || looks_windows_path(configured) {
        path
    } else {
        PathBuf::from(deploy_root).join(path)
    }
}

fn caddy_executable_name() -> &'static str {
    "caddy.exe"
}

struct CommandRunResult {
    success: bool,
    command: String,
    stdout: String,
    stderr: String,
}

fn run_command(command: &Path, args: &[String]) -> CommandRunResult {
    let command_text = command_label(command, args);
    match hidden_command(command).args(args).output() {
        Ok(output) => CommandRunResult {
            success: output.status.success(),
            command: command_text,
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => CommandRunResult {
            success: false,
            command: command_text,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn command_label(command: &Path, args: &[String]) -> String {
    std::iter::once(quote_command_arg(&display_path(command)))
        .chain(args.iter().map(|arg| quote_command_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_command_arg(value: &str) -> String {
    if value.is_empty() || value.chars().any(char::is_whitespace) || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
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
