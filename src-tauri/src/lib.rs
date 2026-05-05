mod commands;
mod domain;
mod runtime;
mod services;
mod storage;
mod telegram;

use chrono::{DateTime, Local};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager};
use zip::ZipArchive;

use crate::domain::*;
use crate::runtime::*;
use crate::services::migration::*;
use crate::storage::*;

pub(crate) fn get_system_status_blocking(app: AppHandle) -> Result<SystemStatus, String> {
    let settings = load_settings(&app)?;
    let state = load_state(&app)?;
    let deploy_root = PathBuf::from(&settings.deploy_root);
    let deploy_root_exists = deploy_root.exists();
    let deploy_root_readonly = fs::metadata(&deploy_root)
        .map(|metadata| metadata.permissions().readonly())
        .ok();
    let node = command_version(node_command(), &["--version"]);
    let pm2 = command_version(pm2_command(), &["--version"]);
    let active_deployment = state
        .active_deployment_id
        .as_ref()
        .and_then(|id| {
            state
                .deployments
                .iter()
                .find(|deployment| &deployment.id == id)
        })
        .cloned();

    Ok(SystemStatus {
        app_version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
        is_windows: cfg!(windows),
        deploy_root: settings.deploy_root,
        deploy_root_exists,
        deploy_root_readonly,
        pm2_execution_enabled: pm2_execution_enabled(),
        node_version: node.ok,
        node_error: node.err,
        pm2_version: pm2.ok,
        pm2_error: pm2.err,
        active_deployment,
    })
}

pub(crate) fn list_software_packages_blocking() -> Result<Vec<SoftwarePackageStatus>, String> {
    Ok(services::software::list_software_packages())
}

pub(crate) fn install_software_package_blocking(
    package_id: String,
) -> Result<SoftwareInstallResult, String> {
    services::software::install_software_package(&package_id)
}

pub(crate) fn get_caddy_status_blocking(app: AppHandle) -> Result<CaddyStatus, String> {
    let settings = load_settings(&app)?;
    Ok(services::caddy::caddy_status(&settings))
}

pub(crate) fn configure_caddy_firewall_blocking() -> Result<CaddyFirewallResult, String> {
    Ok(services::caddy::ensure_caddy_firewall_rules())
}

pub(crate) fn install_caddy_zip_blocking(
    app: AppHandle,
    zip_path: String,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    let settings = save_settings(&app, settings)?;
    services::caddy::install_caddy_zip(&settings, Path::new(&zip_path))
}

pub(crate) fn install_bundled_caddy_blocking(
    app: AppHandle,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    let settings = save_settings(&app, settings)?;
    let zip_path = bundled_caddy_zip_path(&app)?;
    let mut result = services::caddy::install_caddy_zip(&settings, &zip_path)?;
    if result.success {
        result.message = "Bundled Caddy installed.".to_string();
    }
    Ok(result)
}

pub(crate) fn apply_caddy_config_blocking(
    app: AppHandle,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    let settings = save_settings(&app, settings)?;
    services::caddy::apply_caddy_config(&settings)
}

pub(crate) fn apply_caddy_publish_test_config_blocking(
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    let settings = sanitize_settings(settings);
    services::caddy::apply_caddy_publish_test_config(&settings)
}

fn bundled_caddy_zip_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_path = app
        .path()
        .resolve(
            "resources/caddy/caddy_windows_amd64.zip",
            BaseDirectory::Resource,
        )
        .map_err(|err| format!("Failed to resolve bundled Caddy zip resource: {err}"))?;
    if resource_path.is_file() {
        return Ok(resource_path);
    }

    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("caddy")
        .join("caddy_windows_amd64.zip");
    if source_path.is_file() {
        return Ok(source_path);
    }

    Err(format!(
        "Bundled Caddy zip was not found. Checked {} and {}.",
        resource_path.display(),
        source_path.display()
    ))
}

pub(crate) fn run_migration_blocking(app: AppHandle) -> Result<MigrationResult, String> {
    let settings = load_settings(&app)?;
    run_migration_request(&settings)
}

pub(crate) fn list_database_backups_blocking(
    app: AppHandle,
) -> Result<DatabaseBackupListing, String> {
    let settings = load_settings(&app)?;
    list_postgres_backups(&settings)
}

pub(crate) fn run_database_backup_blocking(
    app: AppHandle,
    settings: Settings,
) -> Result<DatabaseCommandResult, String> {
    let settings = save_settings(&app, settings)?;
    run_postgres_backup(&settings)
}

pub(crate) fn restore_database_backup_blocking(
    app: AppHandle,
    backup_path: String,
    settings: Settings,
) -> Result<DatabaseCommandResult, String> {
    let settings = save_settings(&app, settings)?;
    restore_postgres_backup(&settings, Path::new(&backup_path))
}

pub(crate) fn validate_package_blocking(zip_path: String) -> Result<PackageValidation, String> {
    validate_package_path(Path::new(&zip_path))
}

#[derive(Debug, Clone, Default)]
struct DeploymentSourceMetadata {
    release_tag: Option<String>,
    release_asset_name: Option<String>,
    release_digest: Option<String>,
}

pub(crate) fn deploy_package_blocking(
    app: AppHandle,
    zip_path: String,
    settings: Settings,
) -> Result<DeployResult, String> {
    let settings = save_settings(&app, settings)?;
    deploy_package_with_metadata(app, zip_path, settings, DeploymentSourceMetadata::default())
}

pub(crate) fn check_portal_release_blocking(
    app: AppHandle,
    settings: Settings,
) -> Result<PortalReleaseCheckResult, String> {
    let settings = sanitize_settings(settings);
    let state = load_state(&app)?;
    let active_release_tag = state
        .active_deployment_id
        .as_ref()
        .and_then(|id| {
            state
                .deployments
                .iter()
                .find(|deployment| &deployment.id == id)
        })
        .and_then(|deployment| deployment.release_tag.clone());
    services::portal_release::check_latest_release(&settings, active_release_tag)
}

pub(crate) fn deploy_portal_release_blocking(
    app: AppHandle,
    settings: Settings,
) -> Result<DeployResult, String> {
    let settings = save_settings(&app, settings)?;
    emit_deploy_progress(
        &app,
        "download",
        "Download Portal release",
        "running",
        "Downloading latest GitHub release asset.",
    );
    let download = match services::portal_release::download_latest_release(&app, &settings) {
        Ok(download) => download,
        Err(err) => {
            emit_deploy_progress(&app, "download", "Download Portal release", "failed", &err);
            return Err(err);
        }
    };
    emit_deploy_progress(
        &app,
        "download",
        "Download Portal release",
        "done",
        &format!(
            "Downloaded {} to {}.",
            download.release.asset_name,
            services::portal_release::display_download_path(&download)
        ),
    );

    let metadata = DeploymentSourceMetadata {
        release_tag: Some(download.release.tag_name.clone()),
        release_asset_name: Some(download.release.asset_name.clone()),
        release_digest: download.release.asset_digest.clone(),
    };
    let zip_path = download.zip_path.clone();
    let deploy_result = deploy_package_with_metadata(
        app.clone(),
        services::portal_release::display_download_path(&download),
        settings,
        metadata,
    );
    let cleanup_result = cleanup_portal_release_zip(&app, &zip_path);

    match (deploy_result, cleanup_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(deploy_err), Ok(())) => Err(deploy_err),
        (Ok(_), Err(cleanup_err)) => Err(format!(
            "Portal release deployed, but cached release zip cleanup failed: {cleanup_err}"
        )),
        (Err(deploy_err), Err(cleanup_err)) => Err(format!(
            "{deploy_err} Cached release zip cleanup also failed: {cleanup_err}"
        )),
    }
}

fn cleanup_portal_release_zip(app: &AppHandle, zip_path: &Path) -> Result<(), String> {
    emit_deploy_progress(
        app,
        "cleanup",
        "Clean release zip",
        "running",
        "Deleting downloaded GitHub release zip.",
    );

    let result = if zip_path.exists() {
        fs::remove_file(zip_path).map_err(|err| {
            format!(
                "Failed to delete cached Portal release zip {}: {err}",
                zip_path.display()
            )
        })
    } else {
        Ok(())
    };

    match result {
        Ok(()) => {
            emit_deploy_progress(
                app,
                "cleanup",
                "Clean release zip",
                "done",
                "Downloaded release zip deleted.",
            );
            Ok(())
        }
        Err(err) => {
            emit_deploy_progress(app, "cleanup", "Clean release zip", "failed", &err);
            Err(err)
        }
    }
}

fn deploy_package_with_metadata(
    app: AppHandle,
    zip_path: String,
    settings: Settings,
    metadata: DeploymentSourceMetadata,
) -> Result<DeployResult, String> {
    let zip_path_buf = PathBuf::from(&zip_path);
    emit_deploy_progress(
        &app,
        "validate",
        "Validate package",
        "running",
        "Checking package structure.",
    );
    let validation = validate_package_path(&zip_path_buf)?;
    if !validation.valid {
        emit_deploy_progress(
            &app,
            "validate",
            "Validate package",
            "failed",
            "Package is missing required files.",
        );
        return Err(format!(
            "Package is missing required files: {}",
            validation.missing.join(", ")
        ));
    }
    emit_deploy_progress(
        &app,
        "validate",
        "Validate package",
        "done",
        "Required package files found.",
    );

    let deploy_id = format!("deploy_{}", Local::now().format("%Y%m%d_%H%M%S"));
    let deploy_root = PathBuf::from(&settings.deploy_root);
    let deployment_dir = deploy_root.join(&deploy_id);
    emit_deploy_progress(
        &app,
        "extract",
        "Extract release",
        "running",
        &format!("Extracting to {}.", deployment_dir.display()),
    );
    fs::create_dir_all(&deployment_dir).map_err(|err| {
        format!(
            "Failed to create deployment directory {}: {err}",
            deployment_dir.display()
        )
    })?;

    extract_zip(&zip_path_buf, &deployment_dir)?;

    let portal_index = deployment_dir.join("Portal").join("build").join("index.js");
    let webapi_exe = deployment_dir.join("WebApi").join("WebApi.exe");
    if !portal_index.exists() || !webapi_exe.exists() {
        emit_deploy_progress(
            &app,
            "extract",
            "Extract release",
            "failed",
            "Extracted package is missing required files.",
        );
        return Err("Package validation passed but extracted files are missing".to_string());
    }
    emit_deploy_progress(
        &app,
        "extract",
        "Extract release",
        "done",
        "Release extracted successfully.",
    );

    let post_deploy = run_post_deploy_steps(&app, &settings, &deployment_dir)?;

    let logs_dir = deploy_root.join("pm2").join("logs");
    fs::create_dir_all(&logs_dir).map_err(|err| {
        format!(
            "Failed to create PM2 log directory {}: {err}",
            logs_dir.display()
        )
    })?;

    let deployment_path = display_path(&deployment_dir);
    let config_path = deployment_dir.join("config.json");
    emit_deploy_progress(
        &app,
        "config",
        "Generate PM2 config",
        "running",
        &format!("Writing {}.", config_path.display()),
    );
    let config = services::pm2::build_pm2_config(&settings, &deployment_path);
    write_json(&config_path, &config)?;
    emit_deploy_progress(
        &app,
        "config",
        "Generate PM2 config",
        "done",
        "PM2 config written.",
    );

    emit_deploy_progress(
        &app,
        "pm2",
        "Reload PM2",
        "running",
        "Running pm2 startOrReload.",
    );
    let pm2 = services::pm2::run_pm2_config_with_recovery(&config_path, &deployment_path);
    if pm2.attempted && !pm2.success {
        emit_deploy_progress(&app, "pm2", "Reload PM2", "failed", &pm2.message);
        return Err(format!("PM2 reload failed: {}", pm2.message));
    }
    emit_deploy_progress(
        &app,
        "pm2",
        "Reload PM2",
        if pm2.skipped { "skipped" } else { "done" },
        &pm2.message,
    );

    emit_deploy_progress(
        &app,
        "history",
        "Save deployment history",
        "running",
        "Recording active deployment.",
    );
    let mut state = load_state(&app)?;
    let record = DeploymentRecord {
        id: deploy_id,
        created_at: Local::now().to_rfc3339(),
        source_zip: zip_path,
        deployment_path,
        config_path: display_path(&config_path),
        portal_env: settings.portal_env.clone(),
        web_api_env: settings.web_api_env.clone(),
        release_tag: metadata.release_tag,
        release_asset_name: metadata.release_asset_name,
        release_digest: metadata.release_digest,
    };

    state
        .deployments
        .retain(|existing| existing.id != record.id);
    state.deployments.push(record.clone());
    state.active_deployment_id = Some(record.id.clone());
    let removed_deployments = services::pm2::apply_retention(&mut state, settings.retention);
    save_state(&app, &state)?;
    emit_deploy_progress(
        &app,
        "history",
        "Save deployment history",
        "done",
        "Deployment history saved.",
    );

    Ok(DeployResult {
        deployment: record,
        post_deploy,
        pm2,
        removed_deployments,
    })
}

pub(crate) fn list_deployments_blocking(app: AppHandle) -> Result<DeploymentState, String> {
    load_state(&app)
}

pub(crate) fn rollback_deployment_blocking(
    app: AppHandle,
    deployment_id: String,
) -> Result<RollbackResult, String> {
    let mut state = load_state(&app)?;
    let deployment = state
        .deployments
        .iter()
        .find(|deployment| deployment.id == deployment_id)
        .cloned()
        .ok_or_else(|| format!("Deployment not found: {deployment_id}"))?;

    if !PathBuf::from(&deployment.config_path).exists() {
        return Err(format!(
            "Config file no longer exists: {}",
            deployment.config_path
        ));
    }

    let pm2 = services::pm2::run_pm2_config_with_recovery(
        Path::new(&deployment.config_path),
        &deployment.deployment_path,
    );
    if pm2.attempted && !pm2.success {
        return Err(format!("PM2 rollback failed: {}", pm2.message));
    }

    state.active_deployment_id = Some(deployment.id.clone());
    save_state(&app, &state)?;

    Ok(RollbackResult { deployment, pm2 })
}

pub(crate) fn read_log_blocking(
    app: AppHandle,
    app_name: String,
    lines: usize,
) -> Result<LogReadResult, String> {
    let settings = load_settings(&app)?;
    let safe_name = match app_name.as_str() {
        "Portal" => "Portal",
        "WebApi" => "WebApi",
        CADDY_APP_NAME => CADDY_APP_NAME,
        _ => return Err("Unknown app name. Expected Portal, WebApi, or Caddy.".to_string()),
    };
    let line_limit = lines.clamp(50, 2_000);
    let log_path = PathBuf::from(&settings.deploy_root)
        .join("pm2")
        .join("logs")
        .join(format!("{safe_name}.log"));

    let contents = match fs::read_to_string(&log_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(format!("Failed to read log {}: {err}", log_path.display()));
        }
    };

    let all_lines: Vec<String> = contents.lines().map(ToOwned::to_owned).collect();
    let start = all_lines.len().saturating_sub(line_limit);

    Ok(LogReadResult {
        app_name: safe_name.to_string(),
        path: display_path(&log_path),
        lines: all_lines[start..].to_vec(),
    })
}

pub(crate) fn validate_package_path(zip_path: &Path) -> Result<PackageValidation, String> {
    let file = File::open(zip_path)
        .map_err(|err| format!("Failed to open zip package {}: {err}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("Failed to read zip package {}: {err}", zip_path.display()))?;
    let mut entries = BTreeSet::new();

    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|err| format!("Failed to inspect zip entry #{index}: {err}"))?;
        let normalized = normalize_zip_entry(file.name());
        if !normalized.is_empty() {
            entries.insert(normalized);
        }
    }

    let required = [PORTAL_INDEX, WEBAPI_EXE];
    let missing: Vec<String> = required
        .iter()
        .filter(|required_path| !entries.contains(**required_path))
        .map(|required_path| (*required_path).to_string())
        .collect();

    Ok(PackageValidation {
        valid: missing.is_empty(),
        zip_path: display_path(zip_path),
        entries_checked: entries.len(),
        missing,
    })
}

fn normalize_zip_entry(entry: &str) -> String {
    entry
        .replace('\\', "/")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(), String> {
    let file = File::open(zip_path)
        .map_err(|err| format!("Failed to open zip package {}: {err}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("Failed to read zip package {}: {err}", zip_path.display()))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("Failed to read zip entry #{index}: {err}"))?;
        let Some(enclosed_name) = entry.enclosed_name().map(PathBuf::from) else {
            continue;
        };
        let out_path = destination.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|err| {
                format!("Failed to create directory {}: {err}", out_path.display())
            })?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create directory {}: {err}", parent.display()))?;
        }

        let mut out_file = File::create(&out_path)
            .map_err(|err| format!("Failed to create file {}: {err}", out_path.display()))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|err| format!("Failed to extract file {}: {err}", out_path.display()))?;
        out_file
            .flush()
            .map_err(|err| format!("Failed to flush file {}: {err}", out_path.display()))?;
    }

    Ok(())
}

fn run_post_deploy_steps(
    app: &AppHandle,
    settings: &Settings,
    deployment_dir: &Path,
) -> Result<Vec<PostDeployStepResult>, String> {
    let portal_dir = deployment_dir.join("Portal");
    let mut results = Vec::new();

    emit_deploy_progress(
        app,
        "npm",
        "Install Portal dependencies",
        if settings.portal_install_dependencies {
            "running"
        } else {
            "skipped"
        },
        if settings.portal_install_dependencies {
            "Running npm install --omit=dev."
        } else {
            "Portal dependency install is disabled."
        },
    );
    let npm_result =
        run_portal_dependency_install(&portal_dir, settings.portal_install_dependencies);
    match npm_result {
        Ok(result) => {
            emit_deploy_progress(
                app,
                "npm",
                "Install Portal dependencies",
                if result.skipped { "skipped" } else { "done" },
                &result.message,
            );
            results.push(result);
        }
        Err(err) => {
            emit_deploy_progress(app, "npm", "Install Portal dependencies", "failed", &err);
            return Err(err);
        }
    }

    emit_deploy_progress(
        app,
        "asset",
        "Copy Portal assets",
        if settings.portal_asset_copy.enabled {
            "running"
        } else {
            "skipped"
        },
        if settings.portal_asset_copy.enabled {
            "Copying configured Portal asset folder."
        } else {
            "Portal asset copy is disabled."
        },
    );
    let asset_result = run_portal_asset_copy(settings, &portal_dir);
    match asset_result {
        Ok(result) => {
            emit_deploy_progress(
                app,
                "asset",
                "Copy Portal assets",
                if result.skipped { "skipped" } else { "done" },
                &result.message,
            );
            results.push(result);
        }
        Err(err) => {
            emit_deploy_progress(app, "asset", "Copy Portal assets", "failed", &err);
            return Err(err);
        }
    }

    Ok(results)
}

fn emit_deploy_progress(app: &AppHandle, step_id: &str, label: &str, state: &str, detail: &str) {
    let _ = app.emit(
        "deploy-progress",
        DeployProgressEvent {
            step_id: step_id.to_string(),
            label: label.to_string(),
            state: state.to_string(),
            detail: detail.to_string(),
        },
    );
}

fn run_portal_dependency_install(
    portal_dir: &Path,
    enabled: bool,
) -> Result<PostDeployStepResult, String> {
    let command_label = format!("{} install --omit=dev", npm_command());

    if !enabled {
        return Ok(PostDeployStepResult {
            name: "Portal npm install".to_string(),
            success: true,
            skipped: true,
            message: "Portal dependency install is disabled.".to_string(),
            command: Some(command_label),
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let package_json = portal_dir.join("package.json");
    if !package_json.exists() {
        return Err(format!(
            "Portal dependency install is enabled but package.json was not found at {}",
            package_json.display()
        ));
    }

    match hidden_command(npm_command())
        .arg("install")
        .arg("--omit=dev")
        .current_dir(portal_dir)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                Ok(PostDeployStepResult {
                    name: "Portal npm install".to_string(),
                    success: true,
                    skipped: false,
                    message: "Portal production dependencies installed.".to_string(),
                    command: Some(command_label),
                    stdout,
                    stderr,
                })
            } else {
                Err(format!(
                    "Portal npm install failed in {}: {}",
                    portal_dir.display(),
                    if stderr.is_empty() {
                        format!("npm exited with status {}", output.status)
                    } else {
                        stderr
                    }
                ))
            }
        }
        Err(err) => Err(format!(
            "Failed to execute npm install in {}: {err}",
            portal_dir.display()
        )),
    }
}

fn run_portal_asset_copy(
    settings: &Settings,
    portal_dir: &Path,
) -> Result<PostDeployStepResult, String> {
    let config = &settings.portal_asset_copy;
    if !config.enabled {
        return Ok(PostDeployStepResult {
            name: "Portal asset copy".to_string(),
            success: true,
            skipped: true,
            message: "Portal asset copy is disabled.".to_string(),
            command: None,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let source = resolve_asset_source(settings);
    let destination = portal_dir.join(&config.destination);
    if !source.is_dir() {
        return Err(format!(
            "Portal asset copy is enabled but source folder was not found: {}",
            source.display()
        ));
    }

    copy_dir_recursive(&source, &destination)?;

    Ok(PostDeployStepResult {
        name: "Portal asset copy".to_string(),
        success: true,
        skipped: false,
        message: format!(
            "Copied {} to {}.",
            display_path(&source),
            display_path(&destination)
        ),
        command: None,
        stdout: String::new(),
        stderr: String::new(),
    })
}

fn resolve_asset_source(settings: &Settings) -> PathBuf {
    let configured = PathBuf::from(&settings.portal_asset_copy.source);
    if configured.is_absolute() || looks_windows_path(&settings.portal_asset_copy.source) {
        configured
    } else {
        PathBuf::from(&settings.deploy_root).join(configured)
    }
}

fn list_postgres_backups(settings: &Settings) -> Result<DatabaseBackupListing, String> {
    let backup_dir = resolve_database_backup_dir(settings);
    if !backup_dir.exists() {
        return Ok(DatabaseBackupListing {
            backup_dir: display_path(&backup_dir),
            files: Vec::new(),
        });
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&backup_dir).map_err(|err| {
        format!(
            "Failed to read backup directory {}: {err}",
            backup_dir.display()
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "Failed to inspect backup entry under {}: {err}",
                backup_dir.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() || !is_supported_postgres_backup_path(&path) {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|err| format!("Failed to read backup metadata {}: {err}", path.display()))?;
        let modified_at = metadata
            .modified()
            .map(DateTime::<Local>::from)
            .map(|value| value.to_rfc3339())
            .unwrap_or_else(|_| Local::now().to_rfc3339());

        files.push(DatabaseBackupFile {
            file_name: entry.file_name().to_string_lossy().to_string(),
            path: display_path(&path),
            size_bytes: metadata.len(),
            modified_at,
        });
    }

    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(DatabaseBackupListing {
        backup_dir: display_path(&backup_dir),
        files,
    })
}

fn run_postgres_backup(settings: &Settings) -> Result<DatabaseCommandResult, String> {
    let database = &settings.database;
    let backup_dir = resolve_database_backup_dir(settings);
    fs::create_dir_all(&backup_dir).map_err(|err| {
        format!(
            "Failed to create PostgreSQL backup directory {}: {err}",
            backup_dir.display()
        )
    })?;

    let backup_path = backup_dir.join(format!(
        "{}_{}.dump",
        backup_file_prefix(&database.database),
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    let command_path = postgres_command_path(settings, "pg_dump");
    let args = vec![
        "--format=custom".to_string(),
        "--host".to_string(),
        database.host.clone(),
        "--port".to_string(),
        database.port.to_string(),
        "--username".to_string(),
        database.username.clone(),
        "--dbname".to_string(),
        database.database.clone(),
        "--file".to_string(),
        display_path(&backup_path),
        "--no-password".to_string(),
    ];
    let command_label = command_label(&command_path, &args);

    let mut command = hidden_command(&command_path);
    command.args(&args);
    apply_postgres_env(&mut command, database);

    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let success = output.status.success();
            let removed_count = if success {
                apply_database_backup_retention(settings)?
            } else {
                0
            };
            let message = if success {
                let base = format!("PostgreSQL backup created at {}.", backup_path.display());
                if removed_count > 0 {
                    format!("{base} Removed {removed_count} old backup(s).")
                } else {
                    base
                }
            } else if stderr.is_empty() {
                format!("pg_dump exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Ok(DatabaseCommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                path: Some(display_path(&backup_path)),
                stdout,
                stderr,
                message,
            })
        }
        Err(err) => Ok(DatabaseCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            path: Some(display_path(&backup_path)),
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to execute pg_dump: {err}"),
        }),
    }
}

fn restore_postgres_backup(
    settings: &Settings,
    backup_path: &Path,
) -> Result<DatabaseCommandResult, String> {
    if !backup_path.is_file() {
        return Err(format!(
            "PostgreSQL backup file was not found: {}",
            backup_path.display()
        ));
    }
    if !is_supported_postgres_backup_path(backup_path) {
        return Err("Backup file must be .dump, .backup, or .sql.".to_string());
    }

    let database = &settings.database;
    let extension = backup_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let is_plain_sql = extension == "sql";
    let command_path =
        postgres_command_path(settings, if is_plain_sql { "psql" } else { "pg_restore" });
    let args = if is_plain_sql {
        vec![
            "--host".to_string(),
            database.host.clone(),
            "--port".to_string(),
            database.port.to_string(),
            "--username".to_string(),
            database.username.clone(),
            "--dbname".to_string(),
            database.database.clone(),
            "--file".to_string(),
            display_path(backup_path),
            "--no-password".to_string(),
        ]
    } else {
        vec![
            "--clean".to_string(),
            "--if-exists".to_string(),
            "--no-owner".to_string(),
            "--no-privileges".to_string(),
            "--host".to_string(),
            database.host.clone(),
            "--port".to_string(),
            database.port.to_string(),
            "--username".to_string(),
            database.username.clone(),
            "--dbname".to_string(),
            database.database.clone(),
            "--no-password".to_string(),
            display_path(backup_path),
        ]
    };
    let command_label = command_label(&command_path, &args);

    let mut command = hidden_command(&command_path);
    command.args(&args);
    apply_postgres_env(&mut command, database);

    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let success = output.status.success();
            let message = if success {
                format!(
                    "PostgreSQL restore completed for database {}.",
                    database.database
                )
            } else if stderr.is_empty() {
                format!("PostgreSQL restore exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Ok(DatabaseCommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                path: Some(display_path(backup_path)),
                stdout,
                stderr,
                message,
            })
        }
        Err(err) => Ok(DatabaseCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            path: Some(display_path(backup_path)),
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to execute PostgreSQL restore: {err}"),
        }),
    }
}

pub(crate) fn configure_postgres_backup_schedule(
    app: &AppHandle,
    settings: &Settings,
) -> Result<DatabaseCommandResult, String> {
    if !cfg!(windows) {
        return Ok(DatabaseCommandResult {
            attempted: false,
            skipped: true,
            success: true,
            command: "schtasks.exe".to_string(),
            path: None,
            stdout: String::new(),
            stderr: String::new(),
            message: "PostgreSQL backup scheduling is only configured on Windows Server."
                .to_string(),
        });
    }

    if !settings.database.backup_schedule.enabled {
        return unregister_postgres_backup_schedule();
    }

    let script_path = write_postgres_backup_script(app, settings)?;
    let schedule = &settings.database.backup_schedule;
    let task_command = format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        display_path(&script_path)
    );
    let mut args = vec![
        "/Create".to_string(),
        "/TN".to_string(),
        DATABASE_BACKUP_TASK_NAME.to_string(),
        "/TR".to_string(),
        task_command,
        "/SC".to_string(),
        if schedule.frequency == "weekly" {
            "WEEKLY".to_string()
        } else {
            "DAILY".to_string()
        },
    ];
    if schedule.frequency == "weekly" {
        args.extend([
            "/D".to_string(),
            schedule_day_code(&schedule.day_of_week).to_string(),
        ]);
    }
    args.extend(["/ST".to_string(), schedule.time.clone(), "/F".to_string()]);

    let command_label = command_label("schtasks.exe", &args);
    match hidden_command("schtasks.exe").args(&args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let success = output.status.success();
            let cadence = if schedule.frequency == "weekly" {
                format!("weekly on {} at {}", schedule.day_of_week, schedule.time)
            } else {
                format!("daily at {}", schedule.time)
            };
            let message = if success {
                format!("PostgreSQL backup schedule configured {cadence}.")
            } else if stderr.is_empty() {
                format!("schtasks exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Ok(DatabaseCommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                path: Some(display_path(&script_path)),
                stdout,
                stderr,
                message,
            })
        }
        Err(err) => Ok(DatabaseCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            path: Some(display_path(&script_path)),
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to configure Windows scheduled task: {err}"),
        }),
    }
}

fn unregister_postgres_backup_schedule() -> Result<DatabaseCommandResult, String> {
    let args = vec![
        "/Delete".to_string(),
        "/TN".to_string(),
        DATABASE_BACKUP_TASK_NAME.to_string(),
        "/F".to_string(),
    ];
    let command_label = command_label("schtasks.exe", &args);

    match hidden_command("schtasks.exe").args(&args).output() {
        Ok(output) => {
            let mut stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let task_missing = is_missing_scheduled_task_output(&stdout, &stderr);
            let success = output.status.success() || task_missing;
            let message = if task_missing {
                stdout.clear();
                stderr.clear();
                "PostgreSQL backup schedule was already disabled.".to_string()
            } else if success {
                "PostgreSQL backup schedule disabled.".to_string()
            } else if stderr.is_empty() {
                format!("schtasks exited with status {}", output.status)
            } else {
                stderr.clone()
            };

            Ok(DatabaseCommandResult {
                attempted: true,
                skipped: false,
                success,
                command: command_label,
                path: None,
                stdout,
                stderr,
                message,
            })
        }
        Err(err) => Ok(DatabaseCommandResult {
            attempted: true,
            skipped: false,
            success: false,
            command: command_label,
            path: None,
            stdout: String::new(),
            stderr: err.to_string(),
            message: format!("Failed to disable Windows scheduled task: {err}"),
        }),
    }
}

fn is_missing_scheduled_task_output(stdout: &str, stderr: &str) -> bool {
    let output = format!("{stdout}\n{stderr}").to_lowercase();
    output.contains("cannot find")
        || output.contains("not found")
        || output.contains("does not exist")
}

fn write_postgres_backup_script(app: &AppHandle, settings: &Settings) -> Result<PathBuf, String> {
    let script_dir = config_dir(app)?.join("tasks");
    fs::create_dir_all(&script_dir).map_err(|err| {
        format!(
            "Failed to create task script directory {}: {err}",
            script_dir.display()
        )
    })?;
    let script_path = script_dir.join("postgres_backup.ps1");
    let script = build_postgres_backup_script(settings);
    fs::write(&script_path, script).map_err(|err| {
        format!(
            "Failed to write backup script {}: {err}",
            script_path.display()
        )
    })?;
    Ok(script_path)
}

fn build_postgres_backup_script(settings: &Settings) -> String {
    let database = &settings.database;
    let backup_dir = resolve_database_backup_dir(settings);
    let command_path = postgres_command_path(settings, "pg_dump");
    let prefix = backup_file_prefix(&database.database);

    format!(
        r#"$ErrorActionPreference = 'Stop'
$env:PGCONNECT_TIMEOUT = '15'
$env:PGPASSWORD = {password}
$backupDir = {backup_dir}
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
$backupPath = Join-Path $backupDir ({prefix} + '_' + (Get-Date -Format 'yyyyMMdd_HHmmss') + '.dump')
& {command_path} '--format=custom' '--host' {host} '--port' {port} '--username' {username} '--dbname' {database} '--file' $backupPath '--no-password'
if ($LASTEXITCODE -ne 0) {{ exit $LASTEXITCODE }}
Get-ChildItem -Path $backupDir -File | Where-Object {{ @('.dump', '.backup', '.sql') -contains $_.Extension.ToLowerInvariant() }} | Sort-Object LastWriteTime -Descending | Select-Object -Skip {retention} | Remove-Item -Force
"#,
        password = ps_single_quote(&database.password),
        backup_dir = ps_single_quote(&display_path(&backup_dir)),
        prefix = ps_single_quote(&prefix),
        command_path = ps_single_quote(&command_path),
        host = ps_single_quote(&database.host),
        port = ps_single_quote(&database.port.to_string()),
        username = ps_single_quote(&database.username),
        database = ps_single_quote(&database.database),
        retention = database.backup_retention,
    )
}

fn apply_database_backup_retention(settings: &Settings) -> Result<usize, String> {
    let listing = list_postgres_backups(settings)?;
    let mut removed = 0;
    for file in listing
        .files
        .into_iter()
        .skip(settings.database.backup_retention)
    {
        if fs::remove_file(Path::new(&file.path)).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

fn resolve_database_backup_dir(settings: &Settings) -> PathBuf {
    let configured = PathBuf::from(&settings.database.backup_dir);
    if configured.is_absolute() || looks_windows_path(&settings.database.backup_dir) {
        configured
    } else {
        PathBuf::from(&settings.deploy_root).join(configured)
    }
}

fn postgres_command_path(settings: &Settings, command: &str) -> String {
    let executable = if cfg!(windows) {
        format!("{command}.exe")
    } else {
        command.to_string()
    };

    if settings.database.bin_dir.trim().is_empty() {
        executable
    } else {
        display_path(&PathBuf::from(&settings.database.bin_dir).join(executable))
    }
}

fn apply_postgres_env(command: &mut Command, database: &DatabaseSettings) {
    command.env("PGCONNECT_TIMEOUT", "15");
    if !database.password.is_empty() {
        command.env("PGPASSWORD", &database.password);
    }
}

fn is_supported_postgres_backup_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_lowercase())
            .as_deref(),
        Some("dump" | "backup" | "sql")
    )
}

fn backup_file_prefix(value: &str) -> String {
    let prefix: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    if prefix.is_empty() {
        "database".to_string()
    } else {
        prefix
    }
}

fn schedule_day_code(day: &str) -> &'static str {
    match day {
        "Tuesday" => "TUE",
        "Wednesday" => "WED",
        "Thursday" => "THU",
        "Friday" => "FRI",
        "Saturday" => "SAT",
        "Sunday" => "SUN",
        _ => "MON",
    }
}

fn command_label(command: &str, args: &[String]) -> String {
    std::iter::once(quote_command_arg(command))
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

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_dir_all(destination).map_err(|err| {
            format!(
                "Failed to clear destination directory {}: {err}",
                destination.display()
            )
        })?;
    }
    fs::create_dir_all(destination).map_err(|err| {
        format!(
            "Failed to create destination directory {}: {err}",
            destination.display()
        )
    })?;

    for entry in fs::read_dir(source)
        .map_err(|err| format!("Failed to read directory {}: {err}", source.display()))?
    {
        let entry = entry.map_err(|err| {
            format!(
                "Failed to inspect directory entry under {}: {err}",
                source.display()
            )
        })?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "Failed to read file type for {}: {err}",
                entry_path.display()
            )
        })?;

        if file_type.is_dir() {
            copy_dir_recursive(&entry_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&entry_path, &target_path).map_err(|err| {
                format!(
                    "Failed to copy {} to {}: {err}",
                    entry_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            telegram::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_system_status,
            commands::validate_package,
            commands::deploy_package,
            commands::check_portal_release,
            commands::deploy_portal_release,
            commands::list_deployments,
            commands::rollback_deployment,
            commands::read_log,
            commands::run_migration,
            commands::list_database_backups,
            commands::run_database_backup,
            commands::restore_database_backup,
            commands::configure_database_backup_schedule,
            commands::get_caddy_status,
            commands::configure_caddy_firewall,
            commands::install_caddy_zip,
            commands::install_bundled_caddy,
            commands::apply_caddy_config,
            commands::apply_caddy_publish_test_config,
            commands::list_software_packages,
            commands::install_software_package,
            commands::list_pm2_processes,
            commands::control_pm2_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_portal_env_includes_svelte_body_size_limit() {
        let settings = default_settings();
        assert_eq!(
            settings.portal_env.get("BODY_SIZE_LIMIT"),
            Some(&"10M".to_string())
        );
    }

    #[test]
    fn settings_without_portal_release_gets_defaults() {
        let mut value = serde_json::to_value(default_settings()).expect("settings json");
        value
            .as_object_mut()
            .expect("settings object")
            .remove("portalRelease");

        let settings: Settings = serde_json::from_value(value).expect("deserialize settings");
        let settings = sanitize_settings(settings);

        assert!(settings.portal_release.enabled);
        assert_eq!(settings.portal_release.owner, "nguyenhnhatquang");
        assert_eq!(settings.portal_release.repo, "EduPortal_DiemSensei");
        assert_eq!(
            settings.portal_release.asset_name_prefix,
            "EduPortal_DiemSensei_"
        );
        assert_eq!(settings.portal_release.asset_name_suffix, ".zip");
    }

    #[test]
    fn settings_without_telegram_bot_gets_defaults() {
        let mut value = serde_json::to_value(default_settings()).expect("settings json");
        value
            .as_object_mut()
            .expect("settings object")
            .remove("telegramBot");

        let settings: Settings = serde_json::from_value(value).expect("deserialize settings");
        let settings = sanitize_settings(settings);

        assert!(!settings.telegram_bot.enabled);
        assert!(settings.telegram_bot.token.is_empty());
        assert!(settings.telegram_bot.allowed_user_ids.is_empty());
        assert!(settings.telegram_bot.allowed_chat_ids.is_empty());
        assert!(settings.telegram_bot.last_user_id.is_empty());
        assert!(settings.telegram_bot.last_chat_id.is_empty());
    }

    #[test]
    fn deployment_record_without_release_metadata_deserializes() {
        let json = r#"
        {
          "id": "deploy_20260504_120000",
          "createdAt": "2026-05-04T12:00:00+07:00",
          "sourceZip": "C:\\deploy\\package.zip",
          "deploymentPath": "C:\\deploy\\deploy_20260504_120000",
          "configPath": "C:\\deploy\\deploy_20260504_120000\\config.json",
          "portalEnv": {},
          "webApiEnv": {}
        }
        "#;

        let record: DeploymentRecord = serde_json::from_str(json).expect("deployment record");

        assert_eq!(record.release_tag, None);
        assert_eq!(record.release_asset_name, None);
        assert_eq!(record.release_digest, None);
    }

    #[test]
    fn default_database_settings_are_postgresql() {
        let settings = default_settings();
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.database.port, 5432);
        assert_eq!(settings.database.database, "eduportal_control");
        assert_eq!(settings.database.username, "postgres");
        assert_eq!(settings.database.backup_dir, "backups/postgresql");
        assert_eq!(settings.database.backup_retention, 14);
        assert!(!settings.database.backup_schedule.enabled);
        assert_eq!(settings.database.backup_schedule.frequency, "daily");
        assert_eq!(settings.database.backup_schedule.time, "02:00");
    }

    #[test]
    fn default_migration_key_is_not_baked_into_public_builds() {
        let settings = default_settings();
        assert!(settings.migration_url.is_empty());
        assert!(settings.migration_key.is_empty());
    }

    #[test]
    fn sanitizes_database_schedule_and_paths() {
        let mut settings = default_settings();
        settings.database.host = "  ".to_string();
        settings.database.port = 0;
        settings.database.database = " appdb ".to_string();
        settings.database.username = " admin ".to_string();
        settings.database.bin_dir = "relative/bin".to_string();
        settings.database.backup_retention = 999;
        settings.database.backup_schedule.frequency = "monthly".to_string();
        settings.database.backup_schedule.time = "27:99".to_string();
        settings.database.backup_schedule.day_of_week = "Funday".to_string();

        let settings = sanitize_settings(settings);

        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.database.port, 5432);
        assert_eq!(settings.database.database, "appdb");
        assert_eq!(settings.database.username, "admin");
        assert_eq!(settings.database.bin_dir, "");
        assert_eq!(settings.database.backup_retention, 365);
        assert_eq!(settings.database.backup_schedule.frequency, "daily");
        assert_eq!(settings.database.backup_schedule.time, "02:00");
        assert_eq!(settings.database.backup_schedule.day_of_week, "Monday");
    }

    #[test]
    fn postgres_backup_script_uses_whitelisted_dump_command() {
        let mut settings = default_settings();
        settings.database.password = "s'ecret".to_string();
        settings.database.backup_retention = 7;

        let script = build_postgres_backup_script(&settings);

        assert!(script.contains("pg_dump"));
        assert!(script.contains("$env:PGPASSWORD = 's''ecret'"));
        assert!(script.contains("'--format=custom'"));
        assert!(script.contains("Select-Object -Skip 7"));
    }

    #[test]
    fn scheduled_task_missing_output_is_treated_as_absent_task() {
        assert!(is_missing_scheduled_task_output(
            "",
            "ERROR: The system cannot find the file specified."
        ));
        assert!(is_missing_scheduled_task_output(
            "",
            "ERROR: The specified task name \"EduPortal_Control PostgreSQL Backup\" does not exist in the system."
        ));
        assert!(!is_missing_scheduled_task_output(
            "",
            "ERROR: Access is denied."
        ));
    }

    #[test]
    fn pm2_config_uses_windows_paths_and_portal_body_size_limit() {
        let mut settings = default_settings();
        settings.deploy_root = "C:\\deploy".to_string();
        let config =
            services::pm2::build_pm2_config(&settings, "C:\\deploy\\deploy_20260424_095438");
        let portal = &config.apps[0];

        assert_eq!(
            portal.script,
            "C:\\deploy\\deploy_20260424_095438\\Portal\\build\\index.js"
        );
        assert_eq!(portal.log_file, "C:\\deploy\\pm2\\logs\\Portal.log");
        assert_eq!(portal.error_file, "C:\\deploy\\pm2\\logs\\Portal.log");
        assert_ne!(portal.error_file, "NUL");
        assert_eq!(portal.env.get("BODY_SIZE_LIMIT"), Some(&"10M".to_string()));
    }

    #[test]
    fn caddy_pm2_config_uses_separate_error_log() {
        let mut settings = default_settings();
        settings.deploy_root = "C:\\deploy".to_string();
        let config = services::pm2::build_caddy_pm2_config(
            &settings,
            "C:\\deploy\\caddy\\caddy.exe",
            "C:\\deploy\\caddy\\Caddyfile",
            "C:\\deploy\\caddy",
        );
        let caddy = &config.apps[0];

        assert_eq!(caddy.log_file, "C:\\deploy\\pm2\\logs\\Caddy.log");
        assert_eq!(caddy.error_file, "C:\\deploy\\pm2\\logs\\Caddy-error.log");
        assert_ne!(caddy.log_file, caddy.error_file);
    }

    #[test]
    fn pm2_path_matching_handles_windows_separator_variants() {
        assert!(services::pm2::pm2_paths_match(
            "C:/deploy/deploy_20260424_095438/Portal/",
            "c:\\deploy\\deploy_20260424_095438\\Portal"
        ));
    }

    #[test]
    fn package_validation_accepts_required_zip_shape() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let zip_path = temp_dir.path().join("package.zip");
        let file = File::create(&zip_path).expect("zip file");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        writer
            .start_file("Portal/build/index.js", options)
            .expect("portal file");
        writer
            .write_all(b"console.log('portal');")
            .expect("portal contents");
        writer
            .start_file("WebApi/WebApi.exe", options)
            .expect("webapi file");
        writer.write_all(b"binary").expect("webapi contents");
        writer.finish().expect("finish zip");

        let validation = validate_package_path(&zip_path).expect("validation");
        assert!(validation.valid);
        assert!(validation.missing.is_empty());
    }

    #[test]
    fn parses_pm2_jlist_process_shape() {
        let json = r#"
        [
          {
            "name": "Portal",
            "pm_id": 0,
            "pid": 1234,
            "pm2_env": {
              "status": "online",
              "restart_time": 2,
              "unstable_restarts": 0,
              "pm_uptime": 1713940000000,
              "pm_exec_path": "C:\\deploy\\deploy_20260424_095438\\Portal\\build\\index.js",
              "pm_cwd": "C:\\deploy\\deploy_20260424_095438\\Portal"
            },
            "monit": {
              "cpu": 1.5,
              "memory": 52428800
            }
          }
        ]
        "#;

        let processes = services::pm2::parse_pm2_processes(json).expect("pm2 parse");
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].name, "Portal");
        assert_eq!(processes[0].status, "online");
        assert_eq!(processes[0].pid, Some(1234));
        assert_eq!(processes[0].memory, Some(52_428_800));
    }

    #[test]
    fn validates_migration_url_is_localhost_http() {
        let valid = reqwest::Url::parse("http://localhost/migrate").expect("url");
        assert!(validate_local_migration_url(&valid).is_ok());

        let non_localhost = reqwest::Url::parse("http://127.0.0.1/migrate").expect("url");
        assert!(validate_local_migration_url(&non_localhost).is_err());

        let https = reqwest::Url::parse("https://localhost/migrate").expect("url");
        assert!(validate_local_migration_url(&https).is_err());
    }
}
